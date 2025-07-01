use tracing::debug;
use widestring::{U16Str, u16str};
use windows_sys::Win32::{
    Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{EnumThreadWindows, GetClassNameW},
};

use crate::PluginHost;

const IPC_CLASS_PREFIX: &U16Str = u16str!("EVERYTHING_TASKBAR_NOTIFICATION");

struct EnumWindowsData {
    result: Option<IpcWindow>,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = unsafe { &mut *(lparam as *mut EnumWindowsData) };

    let mut buf = [0; 256];
    let len = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if len > 0 {
        let class_name = U16Str::from_slice(&buf[..len as usize]);
        // debug!(?hwnd, ?class_name, "enum_windows_proc");
        if class_name
            .as_slice()
            .starts_with(IPC_CLASS_PREFIX.as_slice())
        {
            data.result = Some(IpcWindow {
                hwnd,
                class_name: class_name.to_string().unwrap(),
            });
            return FALSE;
        }
    }

    TRUE
}

#[derive(Debug)]
pub struct IpcWindow {
    hwnd: HWND,
    class_name: String,
}

impl IpcWindow {
    pub fn from_current_thread() -> Option<Self> {
        let mut data = EnumWindowsData { result: None };

        let tid = unsafe { GetCurrentThreadId() };
        debug!(?tid, "from_current_thread");
        unsafe {
            EnumThreadWindows(tid, Some(enum_windows_proc), &mut data as *mut _ as LPARAM);
        }

        data.result
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn instance_name(&self) -> Option<&str> {
        // e.g. "EVERYTHING_TASKBAR_NOTIFICATION_(1.5a)"
        self.class_name
            .strip_prefix("EVERYTHING_TASKBAR_NOTIFICATION_(")
            .and_then(|s| s.strip_suffix(')'))
    }
}

impl PluginHost {
    pub fn ipc_window_from_main_thread() -> Option<IpcWindow> {
        IpcWindow::from_current_thread()
    }
}
