use core::str;
use std::{
    cell::{Cell, OnceCell, UnsafeCell},
    ffi::{CString, c_void},
    mem,
    ops::Deref,
    slice,
};

use bon::Builder;
use tracing::{debug, trace};

use crate::data::Config;

pub use serde;

pub mod data;
pub mod log;
pub mod macros;
pub mod sys;
pub mod ui;

pub trait PluginApp: 'static {
    type Config: Config;

    fn new(config: Option<Self::Config>) -> Self;

    fn config(&self) -> &Self::Config;

    fn into_config(self) -> Self::Config;
}

/// ## Design
/// - Config may be accessed from multiple threads, and options pages need to modify it. To avoid race conditions, either config is cloned when modifying, and then [`App`] is reloaded with it, i.e. [`arc_swap::ArcSwap`]; or [`App`] is shutdown before modifying and then restarted.
/// - User defined static to work around generic static limit.
///   - Interior mutability to make it easy to use with `static`. But `UnsafeCell` to avoid cost.
///
/// Config lifetime:
/// - May be set with [`PluginHandler::builder()`] (as default value)
/// - May be loaded when [`sys::EVERYTHING_PLUGIN_PM_START`]
/// - Be read when start
/// - Be read when loading (and rendering) options pages ([`sys::EVERYTHING_PLUGIN_PM_LOAD_OPTIONS_PAGE`])
/// - Be written/applied when [`sys::EVERYTHING_PLUGIN_PM_SAVE_OPTIONS_PAGE`], zero, one or multiple times
///   - TODO: Defer
/// - Be saved when [`sys::EVERYTHING_PLUGIN_PM_SAVE_SETTINGS`] (can occur without prior [`sys::EVERYTHING_PLUGIN_PM_SAVE_OPTIONS_PAGE`])
#[derive(Builder)]
pub struct PluginHandler<A: PluginApp> {
    #[builder(skip)]
    host: OnceCell<PluginHost>,

    #[builder(with = |x: impl Into<String>| CString::new(x.into()).unwrap())]
    name: Option<CString>,
    #[builder(with = |x: impl Into<String>| CString::new(x.into()).unwrap())]
    description: Option<CString>,
    #[builder(with = |x: impl Into<String>| CString::new(x.into()).unwrap())]
    author: Option<CString>,
    #[builder(with = |x: impl Into<String>| CString::new(x.into()).unwrap())]
    version: Option<CString>,
    #[builder(with = |x: impl Into<String>| CString::new(x.into()).unwrap())]
    link: Option<CString>,

    #[builder(skip)]
    app: UnsafeCell<Option<A>>,

    #[builder(default)]
    options_pages: Vec<ui::OptionsPage<A>>,
    #[builder(skip)]
    options_message: Cell<ui::OptionsMessage>,
}

unsafe impl<A: PluginApp> Send for PluginHandler<A> {}
unsafe impl<A: PluginApp> Sync for PluginHandler<A> {}

impl<A: PluginApp> PluginHandler<A> {
    /// Not available before handling `EVERYTHING_PLUGIN_PM_INIT`
    pub fn host(&self) -> &PluginHost {
        unsafe { self.host.get().unwrap_unchecked() }
    }

    /// You shouldn't and unlikely need to call this function from multiple threads.
    pub fn handle(&self, msg: u32, data: *mut c_void) -> *mut c_void {
        match msg {
            sys::EVERYTHING_PLUGIN_PM_INIT => {
                #[cfg(feature = "tracing")]
                log::tracing_init();
                debug!("Plugin init");

                _ = self.host.set(unsafe { PluginHost::from_data(data) });

                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_GET_PLUGIN_VERSION => sys::EVERYTHING_PLUGIN_VERSION as _,
            sys::EVERYTHING_PLUGIN_PM_GET_NAME => {
                debug!("Plugin get name");
                match &self.name {
                    Some(name) => name.as_ptr() as _,
                    None => 0 as _,
                }
            }
            sys::EVERYTHING_PLUGIN_PM_GET_DESCRIPTION => {
                debug!("Plugin get description");
                match &self.description {
                    Some(description) => description.as_ptr() as _,
                    None => 0 as _,
                }
            }
            sys::EVERYTHING_PLUGIN_PM_GET_AUTHOR => {
                debug!("Plugin get author");
                match &self.author {
                    Some(author) => author.as_ptr() as _,
                    None => 0 as _,
                }
            }
            sys::EVERYTHING_PLUGIN_PM_GET_VERSION => {
                debug!("Plugin get version");
                match &self.version {
                    Some(version) => version.as_ptr() as _,
                    None => 0 as _,
                }
            }
            sys::EVERYTHING_PLUGIN_PM_GET_LINK => {
                debug!("Plugin get link");
                match &self.link {
                    Some(link) => link.as_ptr() as _,
                    None => 0 as _,
                }
            }
            sys::EVERYTHING_PLUGIN_PM_START => {
                debug!("Plugin start");

                self.app_new(self.load_settings(data));

                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_STOP => {
                debug!("Plugin stop");

                // TODO

                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_UNINSTALL => {
                debug!("Plugin uninstall");

                // TODO

                1 as _
            }
            // Always the last message sent to the plugin
            sys::EVERYTHING_PLUGIN_PM_KILL => {
                debug!("Plugin kill");

                self.app_into_config();

                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_ADD_OPTIONS_PAGES => self.add_options_pages(data),
            sys::EVERYTHING_PLUGIN_PM_LOAD_OPTIONS_PAGE => self.load_options_page(data),
            sys::EVERYTHING_PLUGIN_PM_SAVE_OPTIONS_PAGE => self.save_options_page(data),
            sys::EVERYTHING_PLUGIN_PM_GET_OPTIONS_PAGE_MINMAX => self.get_options_page_minmax(data),
            sys::EVERYTHING_PLUGIN_PM_SIZE_OPTIONS_PAGE => self.size_options_page(data),
            sys::EVERYTHING_PLUGIN_PM_OPTIONS_PAGE_PROC => self.options_page_proc(data),
            sys::EVERYTHING_PLUGIN_PM_KILL_OPTIONS_PAGE => self.kill_options_page(data),
            sys::EVERYTHING_PLUGIN_PM_SAVE_SETTINGS => self.save_settings(data),
            _ => {
                debug!(msg, ?data, "Plugin message");
                0 as _
            }
        }
    }

    fn app_new(&self, config: Option<A::Config>) {
        let app = unsafe { &mut *self.app.get() };
        debug_assert!(app.is_none(), "App already inited");
        *app = Some(A::new(config));
    }

    fn app_into_config(&self) -> A::Config {
        let app = unsafe { &mut *self.app.get() };
        match app.take() {
            Some(app) => app.into_config(),
            None => unreachable!("App not inited"),
        }
    }

    /// Not available during saving config and recreated afterwards. Use [`Self::with_app`] instead when possible.
    pub unsafe fn app(&self) -> &A {
        unsafe { &*self.app.get() }
            .as_ref()
            .expect("App not inited")
    }

    /// Not available during saving config.
    pub fn with_app<T>(&self, f: impl FnOnce(&A) -> T) -> T {
        f(unsafe { self.app() })
    }
}

/// - [ ] `config_*`
/// - [ ] `db_*`
/// - [ ] `debug_*` (tracing)
/// - [ ] `localization_get_*`
/// - [x] `os_enable_or_disable_dlg_item`
/// - [x] `os_get_(local_)?app_data_path_cat_filename`
/// - [x] `plugin_?et_setting_string`
/// - [ ] `property_*`
/// - [x] `ui_options_add_plugin_page`
/// - [x] `utf8_buf_(init|kill)`
/// - [ ] `version_get_*`, `plugin_get_version`
pub struct PluginHost {
    get_proc_address: sys::everything_plugin_get_proc_address_t,
}

impl PluginHost {
    pub fn new(get_proc_address: sys::everything_plugin_get_proc_address_t) -> Self {
        Self { get_proc_address }
    }

    pub unsafe fn from_data(data: *mut c_void) -> Self {
        Self::new(unsafe { mem::transmute(data) })
    }

    fn get_proc_address(
        &self,
    ) -> unsafe extern "system" fn(
        name: *const sys::everything_plugin_utf8_t,
    ) -> *mut ::std::os::raw::c_void {
        unsafe { self.get_proc_address.unwrap_unchecked() }
    }

    pub unsafe fn get<T: Copy>(&self, name: &str) -> Option<T> {
        assert_eq!(mem::size_of::<T>(), mem::size_of::<fn()>());

        trace!(name, "Plugin host get proc address");
        let name = CString::new(name).unwrap();
        let ptr = unsafe { (self.get_proc_address())(name.as_ptr() as _) };
        if ptr.is_null() {
            None
        } else {
            // let f: fn() = unsafe { mem::transmute(ptr) };
            Some(unsafe { mem::transmute_copy(&ptr) })
        }
    }

    /// Initialize a cbuf with an empty string.
    ///
    /// The cbuf must be killed with [`Self::utf8_buf_kill`]
    ///
    /// See also [`Self::utf8_buf_kill`]
    ///
    /// ## Note
    /// Usage:
    /// ```ignore
    /// let mut cbuf = MaybeUninit::uninit();
    /// host.utf8_buf_init(cbuf.as_mut_ptr());
    ///
    /// unsafe { os_get_app_data_path_cat_filename(filename.as_ptr() as _, cbuf.as_mut_ptr()) };
    ///
    /// // Or `utf8_buf_kill()`
    /// self.utf8_buf_into_string(cbuf.as_mut_ptr())
    /// ```
    /// Do not move [`sys::everything_plugin_utf8_buf_t`].
    pub fn utf8_buf_init(&self, cbuf: *mut sys::everything_plugin_utf8_buf_t) {
        let utf8_buf_init: unsafe extern "system" fn(cbuf: *mut sys::everything_plugin_utf8_buf_t) =
            unsafe { self.get("utf8_buf_init") }.unwrap();
        unsafe { utf8_buf_init(cbuf) };
    }

    /// Kill a cbuf initialized with [`Self::utf8_buf_init`].
    ///
    /// Any allocated memory is returned to the system.
    ///
    /// See also [`Self::utf8_buf_init`]
    pub fn utf8_buf_kill(&self, cbuf: *mut sys::everything_plugin_utf8_buf_t) {
        let utf8_buf_kill: unsafe extern "system" fn(cbuf: *mut sys::everything_plugin_utf8_buf_t) =
            unsafe { self.get("utf8_buf_kill") }.unwrap();
        unsafe { utf8_buf_kill(cbuf) };
    }

    pub fn utf8_buf_into_string(&self, cbuf: *mut sys::everything_plugin_utf8_buf_t) -> String {
        let s = unsafe { (*cbuf).to_string() };
        self.utf8_buf_kill(cbuf);
        s
    }
}

impl Deref for sys::everything_plugin_utf8_buf_t {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // str::from_raw_parts(self.buf, self.len)
            str::from_utf8_unchecked(slice::from_raw_parts(self.buf, self.len))
        }
    }
}
