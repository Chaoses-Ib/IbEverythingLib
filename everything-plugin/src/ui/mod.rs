use std::{
    ffi::{CString, c_void},
    mem,
};

use bon::Builder;
use futures_channel::mpsc;
use tracing::{debug, warn};
use windows_sys::Win32::Foundation::HWND;

use crate::{PluginHandler, PluginHost, sys};

#[cfg(feature = "winio")]
pub mod winio;

#[derive(Builder)]
pub struct OptionsPage {
    #[builder(into)]
    name: String,
    #[builder(with = |x: impl FnMut(OptionsPageLoadArgs) -> PageHandle + 'static| Box::new(x))]
    load: Box<dyn FnMut(OptionsPageLoadArgs) -> PageHandle>,
    handle: Option<PageHandle>,
}

#[derive(Debug)]
pub struct OptionsPageLoadArgs {
    parent: HWND,
}

#[derive(Debug)]
pub enum OptionsPageMessage {
    /// `(tx)`
    ///
    /// Just drop the `tx` if there is no need to save the config (i.e. no changes).
    Save(std::sync::mpsc::SyncSender<String>),
    Kill,
}

pub struct PageHandle {
    #[allow(dead_code)]
    thread_handle: std::thread::JoinHandle<()>,
    tx: mpsc::UnboundedSender<OptionsPageMessage>,
}

impl PluginHandler {
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
    pub fn load_options_page(&mut self, data: *mut c_void) -> *mut c_void {
        debug_assert!(!self.options_pages.is_empty());

        let data = unsafe { &mut *(data as *mut sys::everything_plugin_load_options_page_s) };
        {
            let page_hwnd = data.page_hwnd as *const c_void;
            debug!(?page_hwnd, "Plugin load options page");
        }
        let page_hwnd: HWND = unsafe { mem::transmute(data.page_hwnd) };

        let page = &mut self.options_pages[data.user_data as usize];

        page.handle = Some((page.load)(OptionsPageLoadArgs { parent: page_hwnd }));

        1 as _
    }

    #[cfg(feature = "winio")]
    pub fn load_options_page_winio<'a, T: winio::OptionsPageComponent<'a>>(
        &self,
        data: *mut c_void,
    ) -> *mut c_void {
        let data = unsafe { &mut *(data as *mut sys::everything_plugin_load_options_page_s) };
        {
            let page_hwnd = data.page_hwnd as *const c_void;
            debug!(?page_hwnd, "Plugin load options page");
        }
        let page_hwnd: HWND = unsafe { mem::transmute(data.page_hwnd) };

        winio::spawn::<T>(OptionsPageLoadArgs { parent: page_hwnd });

        1 as _
    }

    pub fn save_options_page(&mut self, data: *mut c_void) -> *mut c_void {
        let data = unsafe { &mut *(data as *mut sys::everything_plugin_save_options_page_s) };
        debug!(?data, "Plugin save options page");

        if self.options_pages.is_empty() {
            return 0 as _;
        }

        let page = &mut self.options_pages[data.user_data as usize];
        match page.handle.as_ref() {
            Some(handle) => {
                debug!(is_closed = handle.tx.is_closed(), "Saving options page");

                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                match handle.tx.unbounded_send(OptionsPageMessage::Save(tx)) {
                    Ok(()) => {
                        if let Ok(config) = rx.recv() {
                            debug!(config, "Options page config");
                            self.config = Some(config);
                        }
                    }
                    Err(_) => (),
                }
            }
            None => warn!("Options page handle is None, can't save"),
        }

        0 as _
    }

    pub fn get_options_page_minmax(&mut self, data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn size_options_page(&mut self, data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn options_page_proc(&mut self, data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn kill_options_page(&mut self, data: *mut c_void) -> *mut c_void {
        debug!(?data, "Plugin kill options page");

        if self.options_pages.is_empty() {
            return 0 as _;
        }

        let page = &mut self.options_pages[data as usize];
        match page.handle.take() {
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
