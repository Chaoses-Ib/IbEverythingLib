use everything_plugin::{
    PluginApp, PluginHandler, plugin_main,
    ui::{self, OptionsPage},
};
use serde::{Deserialize, Serialize};

mod widgets;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    s: String,
}

pub struct App {
    config: Config,
}

impl PluginApp for App {
    type Config = Config;

    fn new(config: Option<Self::Config>) -> Self {
        Self {
            config: config.unwrap_or(Config {
                s: "This is an example of multi-line text box.".into(),
            }),
        }
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn into_config(self) -> Self::Config {
        self.config
    }
}

plugin_main!(App, {
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
