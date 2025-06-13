# IbEverythingLib
## [everything-plugin](everything-plugin)
Rust binding for [Everything](https://www.voidtools.com/)'s [plugin SDK](https://www.voidtools.com/forum/viewtopic.php?t=16535).

Features:
- Can make options pages GUI using [Winio](https://github.com/compio-rs/winio) in MVU (Elm) architecture

Example:
```rust
mod options;

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
                    .load(ui::winio::spawn::<options::MainModel>)
                    .build(),
            ])
            .build()
    })
    .handle(msg, data)
}
```

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
