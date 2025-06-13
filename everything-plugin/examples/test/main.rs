use std::ffi::c_void;

use everything_plugin::{
    PluginHandler, handler_or_init,
    ui::{self, OptionsPage},
};

mod widgets;

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
            .options_pages(vec![
                OptionsPage::builder()
                    .name("Test Plugin")
                    // 359 -> 1077 KiB
                    .load(ui::winio::spawn::<widgets::MainModel>)
                    .build(),
                OptionsPage::builder()
                    .name("Test 插件")
                    .load(ui::winio::spawn::<widgets::MainModel>)
                    .build(),
            ])
            .build()
    });
    match msg {
        _ => handler.handle(msg, data),
    }
}
