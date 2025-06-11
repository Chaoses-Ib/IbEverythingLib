use std::{
    cell::OnceCell,
    ffi::{CString, c_void},
    mem,
};

use bon::Builder;
use tracing::{debug, trace};

pub mod sys;

/// A convenient function to initialize [`tracing`] with a default configuration.
#[cfg(feature = "tracing")]
pub fn tracing_init() {
    tracing_subscriber::fmt()
        // TODO: Non-block?
        .with_writer(anstream::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

static mut PLUGIN_HANDLER: OnceCell<PluginHandler> = OnceCell::new();

pub fn handler_init(handler: PluginHandler) {
    _ = unsafe { &*&raw const PLUGIN_HANDLER }.set(handler);
}

pub unsafe fn handler() -> &'static PluginHandler {
    unsafe { (&*&raw const PLUGIN_HANDLER).get().unwrap_unchecked() }
}

pub fn handler_or_init(init: impl FnOnce() -> PluginHandler) -> &'static PluginHandler {
    unsafe { (&*&raw const PLUGIN_HANDLER).get_or_init(init) }
}

#[derive(Builder)]
pub struct PluginHandler {
    #[builder(default)]
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
}

impl PluginHandler {
    /// Not available before handling `EVERYTHING_PLUGIN_PM_INIT`
    pub fn host(&self) -> &PluginHost {
        unsafe { self.host.get().unwrap_unchecked() }
    }

    pub fn handle(&self, msg: u32, data: *mut c_void) -> *mut c_void {
        match msg {
            sys::EVERYTHING_PLUGIN_PM_INIT => {
                #[cfg(feature = "tracing")]
                tracing_init();
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
                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_STOP => {
                debug!("Plugin stop");
                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_UNINSTALL => {
                debug!("Plugin uninstall");
                1 as _
            }
            // Always the last message sent to the plugin
            sys::EVERYTHING_PLUGIN_PM_KILL => {
                debug!("Plugin kill");
                1 as _
            }
            sys::EVERYTHING_PLUGIN_PM_ADD_OPTIONS_PAGES => {
                debug!("Plugin add options pages");
                0 as _
            }
            _ => {
                debug!(msg, ?data, "Plugin message");
                0 as _
            }
        }
    }
}

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

    pub fn get(&self, name: &str) -> Option<*const c_void> {
        trace!(name, "Plugin host get proc address");
        let name = CString::new(name).unwrap();
        let ptr = unsafe { (self.get_proc_address())(name.as_ptr() as _) };
        if ptr.is_null() { None } else { Some(ptr) }
    }

    pub fn ui_options_add_plugin_page(&self, data: *mut c_void, name: &str) {
        let ui_options_add_plugin_page = self.get("ui_options_add_plugin_page").unwrap();
        // Not in header
        let ui_options_add_plugin_page: unsafe extern "system" fn(
            add_custom_page: *mut c_void,
            user_data: *mut c_void,
            name: *const sys::everything_plugin_utf8_t,
        )
            -> *mut ::std::os::raw::c_void = unsafe { mem::transmute(ui_options_add_plugin_page) };
        let name = CString::new(name).unwrap();
        unsafe { ui_options_add_plugin_page(data, 0 as _, name.as_ptr() as _) };
    }
}
