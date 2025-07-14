# IbEverythingLib
## [everything-plugin](everything-plugin)
Rust binding for [Everything](https://www.voidtools.com/)'s [plugin SDK](https://www.voidtools.com/forum/viewtopic.php?t=16535).

Features:
- Load and save config with [Serde](https://github.com/serde-rs/serde)
- Make options pages GUI using [Winio](https://github.com/compio-rs/winio) in MVU (Elm) architecture
- Log with [tracing](https://github.com/tokio-rs/tracing)

Example:
```rust
mod options;

#[derive(Serialize, Deserialize, Debug, Default)]
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
```

Plugins using this library:
- [IbEverythingExt: Everything 拼音搜索、ローマ字検索、快速选择扩展](https://github.com/Chaoses-Ib/IbEverythingExt)

## [everything-cpp](everything-cpp)
A C++17 implementation of [Everything](https://www.voidtools.com/)'s (IPC) SDK.

### Features
- Higher performance. Compared with [the official SDK](https://www.voidtools.com/support/everything/sdk/), it reduces the query time by about 30%.
- Better asynchronous. Its sending blocking time is only 40% of the SDK. And it is based on [`std::future`](https://en.cppreference.com/w/cpp/thread/future.html), which gives you more features about asynchronous.
- Support [named instances](https://www.voidtools.com/en-us/support/everything/multiple_instances/#named_instances).
- Header-only and does not depend on the official DLL.

## See also
Rust bindings (depending on the official DLL) for Everything's (IPC) SDK:
- [reedHam/everything-wrapper: Everything sdk wrapper for rust using bindgen.](https://github.com/reedHam/everything-wrapper)

  [Rust SDK Wrapper - voidtools forum](https://www.voidtools.com/forum/viewtopic.php?t=13256)
- [owtotwo/everything-sdk-rs: An ergonomic Everything(voidtools) SDK wrapper in Rust. (Supports async and raw sdk functions)](https://github.com/owtotwo/everything-sdk-rs)
  - License: GPLv3
- [Ciantic/everything-sys-rs: VoidTools' Everything library as Rust crate](https://github.com/Ciantic/everything-sys-rs/)
