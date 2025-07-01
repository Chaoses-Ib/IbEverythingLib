use everything_plugin::{
    PluginApp, PluginHandler, plugin_main,
    ui::{self, OptionsPage},
};
use serde::{Deserialize, Serialize};

mod options;

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum Mode {
    #[default]
    A,
    B,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    enabled: bool,
    b: bool,
    e: Mode,
    s: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            b: false,
            e: Default::default(),
            s: "Hello, world!".into(),
        }
    }
}

pub struct App {
    config: Config,
}

impl PluginApp for App {
    type Config = Config;

    fn new(config: Option<Self::Config>) -> Self {
        Self {
            config: config.unwrap_or_default(),
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
                .load(ui::winio::spawn::<options::MainModel>)
                .build(),
        ])
        .build()
});
