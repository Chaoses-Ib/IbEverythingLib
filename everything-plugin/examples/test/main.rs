use std::ffi::c_void;

use everything_plugin::{PluginHandler, handler_or_init, sys};

/// - `msg` is a `EVERYTHING_PLUGIN_PM_*` message.
/// - `data` will depend on the `EVERYTHING_PLUGIN_PM_*` message.
///
/// return data based on the `EVERYTHING_PLUGIN_PM_*` message.
#[unsafe(no_mangle)]
pub extern "system" fn everything_plugin_proc(msg: u32, data: *mut c_void) -> *mut c_void {
    let handler = handler_or_init(|| {
        PluginHandler::builder()
            .name("Test Plugin")
            .description("A test plugin for Everything")
            .author("Chaoses-Ib")
            .version("0.1.0")
            .link("https://github.com/Chaoses-Ib/IbEverythingLib")
            .build()
    });
    match msg {
        sys::EVERYTHING_PLUGIN_PM_ADD_OPTIONS_PAGES => {
            handler.handle(msg, data);
            handler
                .host()
                .ui_options_add_plugin_page(data, "Test Plugin");
            handler
                .host()
                .ui_options_add_plugin_page(data, "Test Plugin 2");
            1 as _
        }
        _ => handler.handle(msg, data),
    }
}
