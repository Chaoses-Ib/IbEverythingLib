use std::ffi::CString;

use windows_sys::Win32::{
    Globalization::{GetLocaleInfoW, GetThreadUILanguage, LOCALE_SNAME},
    System::SystemServices::LOCALE_NAME_MAX_LENGTH,
};

use crate::{PluginApp, PluginHandler, PluginHost, sys};

impl<A: PluginApp> PluginHandler<A> {
    pub fn get_language_name(&self) -> String {
        match self.get_host() {
            Some(host) => host.config_get_language_name(),
            None => PluginHost::get_thread_language_name(),
        }
    }
}

impl PluginHost {
    /// Get an Everything setting value by setting name.
    ///
    /// Returns the integer value of the setting.
    ///
    /// ## Example
    /// ```ignore
    /// let app_data = host.config_get_int_value("app_data");
    /// ```
    pub fn config_get_int_value(&self, name: &str) -> i32 {
        let config_get_int_value: unsafe extern "system" fn(
            name: *const sys::everything_plugin_utf8_t,
        ) -> i32 = unsafe { self.get("config_get_int_value").unwrap_unchecked() };
        let name = CString::new(name).unwrap();
        unsafe { config_get_int_value(name.as_ptr() as _) }
    }

    /// Set an Everything setting value by setting name.
    ///
    /// Returns 1 if the setting was changed.  
    /// Returns 0 if the setting remains the same.
    ///
    /// ## Example
    /// ```ignore
    /// host.config_set_int_value("always_keep_sort", 1);
    /// ```
    pub fn config_set_int_value(&self, name: &str, value: i32) -> i32 {
        let config_set_int_value: unsafe extern "system" fn(
            name: *const sys::everything_plugin_utf8_t,
            value: i32,
        ) -> i32 = unsafe { self.get("config_set_int_value").unwrap_unchecked() };
        let name = CString::new(name).unwrap();
        unsafe { config_set_int_value(name.as_ptr() as _, value) }
    }

    /// ## Returns
    /// - `None`: User Default
    /// - `Some(u16)`: Language identifier
    pub fn config_get_language(&self) -> Option<u16> {
        match self.config_get_int_value("language") {
            0 => None,
            id => Some(id as u16),
        }
    }

    pub fn get_language_name(language: u16) -> String {
        let mut lcdata = [0u16; LOCALE_NAME_MAX_LENGTH as usize];
        _ = unsafe {
            GetLocaleInfoW(
                language as u32,
                LOCALE_SNAME,
                lcdata.as_mut_ptr(),
                size_of_val(&lcdata) as i32,
            )
        };
        String::from_utf16_lossy(&lcdata)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn get_thread_language_name() -> String {
        let language = unsafe { GetThreadUILanguage() };
        Self::get_language_name(language)
    }

    pub fn config_get_language_name(&self) -> String {
        let language = self
            .config_get_language()
            .unwrap_or_else(|| unsafe { GetThreadUILanguage() });
        Self::get_language_name(language)
    }
}
