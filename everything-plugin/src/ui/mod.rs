use std::{
    cell::UnsafeCell,
    ffi::{CString, c_void},
    fmt::Debug,
    mem,
};

use bon::Builder;
use futures_channel::mpsc;
use tracing::{debug, warn};
use windows_sys::Win32::Foundation::HWND;

use crate::{PluginApp, PluginHandler, PluginHost, sys};

#[cfg(feature = "winio")]
pub mod winio;

#[derive(Builder)]
pub struct OptionsPage<A: PluginApp> {
    #[builder(into)]
    name: String,
    #[builder(with = |x: impl FnMut(OptionsPageLoadArgs) -> PageHandle<A> + 'static| UnsafeCell::new(Box::new(x)))]
    load: UnsafeCell<Box<dyn FnMut(OptionsPageLoadArgs) -> PageHandle<A>>>,
    #[builder(default)]
    handle: UnsafeCell<Option<PageHandle<A>>>,
}

impl<A: PluginApp> OptionsPage<A> {
    fn load_mut(&self) -> &mut dyn FnMut(OptionsPageLoadArgs) -> PageHandle<A> {
        unsafe { &mut *self.load.get() }
    }

    fn handle(&self) -> &Option<PageHandle<A>> {
        unsafe { &*self.handle.get() }
    }

    fn handle_mut(&self) -> &mut Option<PageHandle<A>> {
        unsafe { &mut *self.handle.get() }
    }
}

#[derive(Debug)]
pub struct OptionsPageLoadArgs {
    parent: HWND,
}

pub enum OptionsPageMessage<A: PluginApp> {
    /// `(config, tx)`
    ///
    /// Just drop the `tx` if there is no need to save the config (i.e. no changes).
    ///
    /// Note [`PluginHandler::app`] is not available during saving.
    Save(
        &'static mut A::Config,
        std::sync::mpsc::SyncSender<&'static mut A::Config>,
    ),
    Kill,
}

impl<A: PluginApp> Debug for OptionsPageMessage<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionsPageMessage::Save(_config, _tx) => write!(f, "OptionsPageMessage::Save"),
            OptionsPageMessage::Kill => write!(f, "OptionsPageMessage::Kill"),
        }
    }
}

pub struct PageHandle<A: PluginApp> {
    #[allow(dead_code)]
    thread_handle: std::thread::JoinHandle<()>,
    tx: mpsc::UnboundedSender<OptionsPageMessage<A>>,
}

impl<A: PluginApp> PluginHandler<A> {
    pub fn add_options_pages(&self, data: *mut c_void) -> *mut c_void {
        debug!("Plugin add options pages");
        if self.options_pages.is_empty() {
            0 as _
        } else {
            for (i, page) in self.options_pages.iter().enumerate() {
                self.host()
                    .ui_options_add_plugin_page(data, i as _, &page.name);
            }
            1 as _
        }
    }

    /// Evertyhing only loads a options page when the user selects it
    ///
    /// TODO: Enable Apply
    /// TODO: `tooltip_hwnd`
    pub fn load_options_page(&self, data: *mut c_void) -> *mut c_void {
        debug_assert!(!self.options_pages.is_empty());

        let data = unsafe { &mut *(data as *mut sys::everything_plugin_load_options_page_s) };
        {
            let page_hwnd = data.page_hwnd as *const c_void;
            debug!(?page_hwnd, "Plugin load options page");
        }
        let page_hwnd: HWND = unsafe { mem::transmute(data.page_hwnd) };

        let page = &self.options_pages[data.user_data as usize];

        *page.handle_mut() = Some((page.load_mut())(OptionsPageLoadArgs { parent: page_hwnd }));

        1 as _
    }

    #[cfg(feature = "winio")]
    pub fn load_options_page_winio<'a, T: winio::OptionsPageComponent<'a, A>>(
        &self,
        data: *mut c_void,
    ) -> *mut c_void {
        let data = unsafe { &mut *(data as *mut sys::everything_plugin_load_options_page_s) };
        {
            let page_hwnd = data.page_hwnd as *const c_void;
            debug!(?page_hwnd, "Plugin load options page");
        }
        let page_hwnd: HWND = unsafe { mem::transmute(data.page_hwnd) };

        winio::spawn::<A, T>(OptionsPageLoadArgs { parent: page_hwnd });

        1 as _
    }

    pub fn save_options_page(&self, data: *mut c_void) -> *mut c_void {
        let data = unsafe { &mut *(data as *mut sys::everything_plugin_save_options_page_s) };
        debug!(?data, "Plugin save options page");

        if self.options_pages.is_empty() {
            return 0 as _;
        }

        let page = &self.options_pages[data.user_data as usize];
        match page.handle() {
            Some(handle) => {
                debug!(is_closed = handle.tx.is_closed(), "Saving options page");

                let (tx, rx) = std::sync::mpsc::sync_channel(1);

                let mut config = self.app_into_config();
                let config_static: &'static mut A::Config = unsafe { mem::transmute(&mut config) };
                match handle
                    .tx
                    .unbounded_send(OptionsPageMessage::Save(config_static, tx))
                {
                    Ok(()) => {
                        if let Ok(_config) = rx.recv() {
                            debug!(?config, "Options page config");
                            self.app_new(Some(config));
                        }
                    }
                    Err(_) => (),
                }
            }
            None => warn!("Options page handle is None, can't save"),
        }

        0 as _
    }

    pub fn get_options_page_minmax(&self, _data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn size_options_page(&self, _data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn options_page_proc(&self, _data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn kill_options_page(&self, data: *mut c_void) -> *mut c_void {
        debug!(?data, "Plugin kill options page");

        if self.options_pages.is_empty() {
            return 0 as _;
        }

        let page = &self.options_pages[data as usize];
        match page.handle_mut().take() {
            Some(handle) => {
                debug!(is_closed = handle.tx.is_closed(), "Killing options page");
                _ = handle.tx.unbounded_send(OptionsPageMessage::Kill);
                #[cfg(debug_assertions)]
                std::thread::spawn(|| {
                    // debug: ~14ms
                    handle.thread_handle.join().unwrap();
                    debug!("Options page thread finished");
                });
            }
            None => warn!("Options page handle is None, can't kill"),
        }
        1 as _
    }
}

impl PluginHost {
    pub fn ui_options_add_plugin_page(
        &self,
        data: *mut c_void,
        user_data: *mut c_void,
        name: &str,
    ) {
        // Not in header
        let ui_options_add_plugin_page: unsafe extern "system" fn(
            add_custom_page: *mut c_void,
            user_data: *mut c_void,
            name: *const sys::everything_plugin_utf8_t,
        )
            -> *mut ::std::os::raw::c_void =
            unsafe { self.get("ui_options_add_plugin_page") }.unwrap();
        let name = CString::new(name).unwrap();
        unsafe { ui_options_add_plugin_page(data, user_data, name.as_ptr() as _) };
    }
}
