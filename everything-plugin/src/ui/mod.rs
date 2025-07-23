//! ## Everything plugin SDK GUI API
//! Not implemented, just for comparison:
//! - Button, Checkbox, Edit, GroupBox, Listbox, NumberEdit, PasswordEdit, Static, Tooltip
//! - set_text, enable, redraw, Listbox API
//! - File dialogs
//! - No dark mode support

use std::{
    cell::UnsafeCell,
    ffi::{CString, c_void},
    fmt::Debug,
    mem,
};

use bon::Builder;
use futures_channel::mpsc;
use tracing::{debug, trace, warn};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GA_ROOT, GetAncestor, SWP_NOZORDER, SendMessageW, SetWindowPos, WM_CLOSE, WM_CREATE,
        WM_CTLCOLORDLG, WM_MOVE, WM_PARENTNOTIFY, WM_SIZE,
    },
};

use crate::{PluginApp, PluginHandler, PluginHost, sys};

#[cfg(feature = "winio")]
pub mod winio;

/// TODO: Icon?
/// TODO: Share one runtime
#[derive(Builder)]
pub struct OptionsPage<A: PluginApp> {
    /// If conflicts with other plugins, Everything will append a " (plugin.dll)" suffix.
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

enum OptionsPageInternalMessage<A: PluginApp> {
    Msg(OptionsPageMessage<A>),
    Size((i32, i32)),
    /// Map to [`WM_CLOSE`] to reuse `WindowEvent::Close`
    Kill,
}

impl<A: PluginApp> From<OptionsPageMessage<A>> for OptionsPageInternalMessage<A> {
    fn from(msg: OptionsPageMessage<A>) -> Self {
        OptionsPageInternalMessage::Msg(msg)
    }
}

impl<A: PluginApp> OptionsPageInternalMessage<A> {
    pub fn try_into(self, window: HWND) -> Option<OptionsPageMessage<A>> {
        match self {
            OptionsPageInternalMessage::Msg(msg) => Some(msg),
            OptionsPageInternalMessage::Size(v) => {
                debug!(?v, "OptionsPageInternalMessage::Size");
                // We do not use `SWP_NOMOVE` to mitigate the occasional misplacement bug by the way, see `winio::adjust_window` for details.
                unsafe { SetWindowPos(window, 0 as _, 0, 0, v.0, v.1, SWP_NOZORDER) };
                None
            }
            OptionsPageInternalMessage::Kill => {
                unsafe { SendMessageW(window, WM_CLOSE, 0, 0) };
                None
            }
        }
    }
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
}

impl<A: PluginApp> Debug for OptionsPageMessage<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionsPageMessage::Save(_config, _tx) => write!(f, "OptionsPageMessage::Save"),
        }
    }
}

pub struct PageHandle<A: PluginApp> {
    #[allow(dead_code)]
    thread_handle: std::thread::JoinHandle<()>,
    tx: mpsc::UnboundedSender<OptionsPageInternalMessage<A>>,
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

        // Enable Apply button
        // Only works when switching to the page after loading the options window
        // self.host().ui_options_enable_or_disable_apply_button(
        //     self.host().ui_options_from_page_hwnd(page_hwnd),
        //     true,
        // );

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
                    .unbounded_send(OptionsPageMessage::Save(config_static, tx).into())
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

        // data.enable_apply = 1;
        self.options_message.set(OptionsMessage::EnableApply(true));

        1 as _
    }

    pub fn get_options_page_minmax(&self, _data: *mut c_void) -> *mut c_void {
        // TODO
        0 as _
    }

    pub fn size_options_page(&self, _data: *mut c_void) -> *mut c_void {
        // We listen to WM_SIZE in options_page_proc instead
        0 as _
    }

    pub fn options_page_proc(&self, data: *mut c_void) -> *mut c_void {
        let data = unsafe { &mut *(data as *mut sys::everything_plugin_options_page_proc_s) };
        trace!(?data, "Plugin options page proc");

        if self.options_pages.is_empty() {
            return 0 as _;
        }
        let page = &self.options_pages[data.user_data as usize];

        let msg = data.msg as u32;
        let w_param = data.wParam;
        let l_param = data.lParam;

        let options_hwnd = unsafe { mem::transmute(data.options_hwnd) };
        // let page_hwnd = unsafe { mem::transmute(data.page_hwnd) };
        // debug_assert_eq!(
        //     self.host().ui_options_from_page_hwnd(page_hwnd),
        //     options_hwnd
        // );

        // Plugin add options pages
        // msg: WM_SHOWWINDOW (24), wParam: 1, lParam: 0, page_hwnd: 0x7e1b46
        // msg: WM_WINDOWPOSCHANGING (70), wParam: 0, lParam: 15718896
        // msg: WM_WINDOWPOSCHANGED (71), wParam: 0, lParam: 15718896
        // msg: WM_WINDOWPOSCHANGING (70), wParam: 0, lParam: 15719520
        // Plugin load options page page_hwnd=0x7e1b46
        // msg: WM_PARENTNOTIFY (528), wParam: 1, lParam: 597884
        // msg: WM_WINDOWPOSCHANGING (70), wParam: 0, lParam: 15718928
        // msg: WM_NCCALCSIZE (131), wParam: 1, lParam: 15718880
        // if direct: msg: WM_NEXTDLGCTL (40), wParam: 6819452, lParam: 1
        // msg: WM_NCPAINT (133), wParam: 1, lParam: 0
        // msg: WM_ERASEBKGND (20), wParam: 1879128502, lParam: 0
        // msg: WM_WINDOWPOSCHANGED (71), wParam: 0, lParam: 15718928
        // msg: WM_SIZE (5), wParam: 0, lParam: 39781242
        // msg: WM_NCPAINT (133), wParam: 1, lParam: 0
        // msg: WM_ERASEBKGND (20), wParam: 536950527, lParam: 0
        // msg: WM_CTLCOLORDLG (310), wParam: 536950527, lParam: 8264518
        // msg: WM_WINDOWPOSCHANGING (70), wParam: 0, lParam: 15716848
        // msg: WM_PAINT (15), wParam: 0, lParam: 0
        // msg: WM_WINDOWPOSCHANGING (70), wParam: 0, lParam: 15716848
        // msg: WM_PAINT (15), wParam: 0, lParam: 0
        match msg {
            WM_MOVE | WM_CLOSE => {
                debug!(
                    msg,
                    lParam = ?l_param as *const c_void,
                    lParam = ?w_param as *const c_void,
                );
            }
            WM_SIZE => {
                if let Some(handle) = page.handle() {
                    _ = handle.tx.unbounded_send(OptionsPageInternalMessage::Size((
                        (l_param & 0xFFFF) as i32,
                        (l_param >> 16) as i32,
                    )));
                }
            }
            WM_PARENTNOTIFY => {
                debug!(wParam = data.wParam, "WM_PARENTNOTIFY");
                match data.wParam as u32 {
                    WM_CREATE => {
                        // Only works when switching to the page after loading the options window
                        // self.host()
                        //     .ui_options_enable_or_disable_apply_button(options_hwnd, true);
                    }
                    _ => (),
                }
            }
            WM_CTLCOLORDLG => {
                debug!(lParam = ?data.lParam as *const c_void, "WM_CTLCOLORDLG");
                self.host()
                    .ui_options_enable_or_disable_apply_button(options_hwnd, true);
            }
            _ => (),
        }

        match self.options_message.take() {
            OptionsMessage::Noop => (),
            OptionsMessage::EnableApply(enable) => self
                .host()
                .ui_options_enable_or_disable_apply_button(options_hwnd, enable),
        }

        1 as _
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
                _ = handle.tx.unbounded_send(OptionsPageInternalMessage::Kill);
                // TODO: Without waiting can cause dangling handle
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

#[derive(Default)]
pub enum OptionsMessage {
    #[default]
    Noop,
    EnableApply(bool),
}

/// `options_hwnd`
#[repr(i32)]
pub enum OptionsDlgItem {
    ApplyButton = 1001,
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
            unsafe { self.get("ui_options_add_plugin_page").unwrap_unchecked() };
        let name = CString::new(name).unwrap();
        unsafe { ui_options_add_plugin_page(data, user_data, name.as_ptr() as _) };
    }

    pub fn ui_options_from_page_hwnd(page_hwnd: HWND) -> HWND {
        unsafe { GetAncestor(page_hwnd, GA_ROOT) }
    }

    pub fn ui_options_enable_or_disable_apply_button(&self, options_hwnd: HWND, enable: bool) {
        self.os_enable_or_disable_dlg_item(
            options_hwnd,
            OptionsDlgItem::ApplyButton as i32,
            enable,
        );
    }

    /// Enable or disable a dialog control.
    ///
    /// ## Note
    /// For `options_hwnd`, see [`OptionsDlgItem`].
    pub fn os_enable_or_disable_dlg_item(&self, parent_hwnd: HWND, id: i32, enable: bool) {
        let os_enable_or_disable_dlg_item: unsafe extern "system" fn(
            parent_hwnd: HWND,
            id: i32,
            enable: i32,
        ) = unsafe { self.get("os_enable_or_disable_dlg_item").unwrap_unchecked() };
        unsafe { os_enable_or_disable_dlg_item(parent_hwnd, id, enable as i32) };
    }
}
