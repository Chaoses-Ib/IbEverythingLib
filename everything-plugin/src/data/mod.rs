//! Everything data directory:
//! - `app_data=1`: `%APPDATA%\Everything`, `%LOCALAPPDATA%\Everything` (also Scoop)
//! - `app_data=0`: parent directory of `Everything.exe`/`Everything64.exe`
//!
//! Either `plugin_?et_setting_string` or `os_get_(local_)?app_data_path_cat_filename` can be used to read/write configs:
//! - `plugin_?et_setting_string`:
//!   - Depends on plugin message data
//!   - Is in ini format
//!   - Must use DLL name as the section and limited to that section
//!   - Has built-in backup mechanism
//! - `os_get_(local_)?app_data_path_cat_filename`:
//!   - Is more flexible
//!   - Can also be used to read/write general data
//!   - But be careful with named instances

use std::{
    ffi::{CStr, CString, c_void},
    fmt::Debug,
    mem::MaybeUninit,
    path::PathBuf,
};

use serde::{Serialize, de::DeserializeOwned};
use tracing::{debug, error};

use crate::{PluginApp, PluginHandler, PluginHost, sys};

pub mod config;

pub trait Config: Serialize + DeserializeOwned + Send + Debug + 'static {}

impl<T: Serialize + DeserializeOwned + Send + Debug + 'static> Config for T {}

impl<A: PluginApp> PluginHandler<A> {
    pub fn load_settings(&self, data: *mut c_void) -> Option<A::Config> {
        match self.get_host() {
            Some(host) => {
                let s = host.plugin_get_setting_string(data, "_", 0 as _);
                if !s.is_null() {
                    let config = unsafe { CStr::from_ptr(s as _) };
                    debug!(
                        config = %config.to_str().unwrap_or("Invalid UTF-8"),
                        "Plugin config",
                    );
                    match serde_json::from_slice(config.to_bytes()) {
                        Ok(config) => Some(config),
                        Err(e) => {
                            error!(%e, "Plugin config parse error");
                            None
                        }
                    }
                } else {
                    None
                }
            }
            None if !data.is_null() => {
                // TODO: unstable Box::into_inner()
                let config = *unsafe { Box::from_raw(data as *mut A::Config) };
                debug!(?config, "Plugin config");
                Some(config)
            }
            None => None,
        }
    }

    pub fn save_settings(&self, data: *mut c_void) -> *mut c_void {
        let config = unsafe { self.app() }.config();
        let config = serde_json::to_string(config).unwrap();
        debug!(%config, "Plugin save settings");

        self.host().plugin_set_setting_string(data, "_", &config);
        1 as _
    }
}

impl PluginHost {
    fn os_get_app_data_path_cat_filename_common(&self, name: &str, filename: &str) -> PathBuf {
        let os_get_app_data_path_cat_filename: unsafe extern "system" fn(
            filename: *const sys::everything_plugin_utf8_t,
            cbuf: *mut sys::everything_plugin_utf8_buf_t,
        ) = unsafe { self.get(name).unwrap_unchecked() };

        let filename = CString::new(filename).unwrap();

        let mut cbuf = MaybeUninit::uninit();
        self.utf8_buf_init(cbuf.as_mut_ptr());

        unsafe { os_get_app_data_path_cat_filename(filename.as_ptr() as _, cbuf.as_mut_ptr()) };

        self.utf8_buf_into_string(cbuf.as_mut_ptr()).into()
    }

    pub fn os_get_app_data_path(&self) -> PathBuf {
        self.os_get_app_data_path_cat_filename("")
    }

    /// Build the setting or data full path using the specified filename.
    ///
    /// The full path is stored in cbuf.  
    /// This will either be `%APPDATA%\Everything\filename` or filename in the same location as your `Everything.exe`  
    /// Depending on your app_data setting.  
    /// cbuf must be initialized with [`Self::utf8_buf_init`]
    ///
    /// See also [`Self::utf8_buf_init`]
    pub fn os_get_app_data_path_cat_filename(&self, filename: &str) -> PathBuf {
        self.os_get_app_data_path_cat_filename_common("os_get_app_data_path_cat_filename", filename)
    }

    pub fn os_get_local_app_data_path(&self) -> PathBuf {
        self.os_get_local_app_data_path_cat_filename("")
    }

    /// Build the data full path using the specified filename.
    ///
    /// The full path is stored in cbuf.  
    /// This will either be `%LOCALAPPDATA%\Everything\filename` or filename in the same location as your `Everything.exe`  
    /// Depending on your app_data setting.
    pub fn os_get_local_app_data_path_cat_filename(&self, filename: &str) -> PathBuf {
        self.os_get_app_data_path_cat_filename_common(
            "os_get_local_app_data_path_cat_filename",
            filename,
        )
    }

    /// Get an string setting value by name from the specified setting sorted list.
    ///
    /// Returns a pointer to the string value.
    ///
    /// `current_value` is returned if the setting value was not found.
    ///
    /// ## Note
    /// - `data`: On [`sys::EVERYTHING_PLUGIN_PM_START`]
    pub fn plugin_get_setting_string(
        &self,
        data: *mut c_void,
        name: &str,
        current_string: *mut sys::everything_plugin_utf8_t,
    ) -> *mut sys::everything_plugin_utf8_t {
        // Not in header
        let plugin_get_setting_string: unsafe extern "system" fn(
            sorted_list: *mut c_void,
            name: *const sys::everything_plugin_utf8_t,
            current_string: *mut sys::everything_plugin_utf8_t,
        ) -> *mut sys::everything_plugin_utf8_t =
            unsafe { self.get("plugin_get_setting_string").unwrap_unchecked() };
        let name = CString::new(name).unwrap();
        unsafe { plugin_get_setting_string(data, name.as_ptr() as _, current_string) }
    }

    /// Writes a string setting value with the specified name to the specified output stream.
    ///
    /// ## Note
    /// - `data`: On [`sys::EVERYTHING_PLUGIN_PM_SAVE_SETTINGS`]
    /// - `value`: Must be single-line. Chars after the first newline cannot be read.
    ///
    /// `Plugins{-instance_name}.ini`:
    /// ```ini
    /// [{plugin_dll}]
    /// {name}={value}
    /// ...
    /// ```
    pub fn plugin_set_setting_string(&self, data: *mut c_void, name: &str, value: &str) {
        debug_assert!(
            !value.contains('\n'),
            "setting string value must be single-line"
        );

        // Not in header
        let plugin_set_setting_string: unsafe extern "system" fn(
            output_stream: sys::everything_plugin_output_stream_t,
            name: *const sys::everything_plugin_utf8_t,
            value: *const sys::everything_plugin_utf8_t,
        ) = unsafe { self.get("plugin_set_setting_string").unwrap_unchecked() };
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        unsafe { plugin_set_setting_string(data, name.as_ptr() as _, value.as_ptr() as _) };
    }

    /// Non-official `plugins.json` path.
    ///
    /// TODO: Named instances
    pub fn plugin_setting_json_path(&self) -> PathBuf {
        self.os_get_app_data_path().join("plugins.json")
    }
}
