//! Rust binding for Everything's IPC SDK.
//!
//! ## Features
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "doc", doc = document_features::document_features!())]

use tracing::debug;
use widestring::{U16Str, u16str};
use windows_sys::Win32::{
    Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{EnumThreadWindows, GetClassNameW, SendMessageW, WM_USER},
};

const IPC_CLASS_PREFIX: &U16Str = u16str!("EVERYTHING_TASKBAR_NOTIFICATION");

const EVERYTHING_WM_IPC: u32 = WM_USER;

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

    pub fn get_version(&self) -> Version {
        const EVERYTHING_IPC_GET_MAJOR_VERSION: u32 = 0;
        const EVERYTHING_IPC_GET_MINOR_VERSION: u32 = 1;
        const EVERYTHING_IPC_GET_REVISION: u32 = 2;
        const EVERYTHING_IPC_GET_BUILD_NUMBER: u32 = 3;
        // const EVERYTHING_IPC_GET_TARGET_MACHINE: u32 = 5;

        let send_u32 = |command: u32| unsafe {
            SendMessageW(self.hwnd, EVERYTHING_WM_IPC, command as usize, 0)
        } as u32;

        Version {
            major: send_u32(EVERYTHING_IPC_GET_MAJOR_VERSION),
            minor: send_u32(EVERYTHING_IPC_GET_MINOR_VERSION),
            revision: send_u32(EVERYTHING_IPC_GET_REVISION),
            build: send_u32(EVERYTHING_IPC_GET_BUILD_NUMBER),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
    pub build: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, revision: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            revision,
            build,
        }
    }
}
