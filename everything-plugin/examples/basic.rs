use std::ffi::c_void;

use everything_plugin::{
    PluginHandler, handler_or_init,
    ui::{self, OptionsPage},
};

#[path = "test/widgets.rs"]
mod widgets;

#[unsafe(no_mangle)]
pub extern "system" fn everything_plugin_proc(msg: u32, data: *mut c_void) -> *mut c_void {
    handler_or_init(|| {
        PluginHandler::builder()
            .name("Test Plugin")
            .description("A test plugin for Everything")
            .author("Chaoses-Ib")
            .version("0.1.0")
            .link("https://github.com/Chaoses-Ib/IbEverythingLib")
            .options_pages(vec![
                OptionsPage::builder()
                    .name("Test Plugin")
                    .load(ui::winio::spawn::<widgets::MainModel>)
                    .build(),
            ])
            .build()
    })
    .handle(msg, data)
}
