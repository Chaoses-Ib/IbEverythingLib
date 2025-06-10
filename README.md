# IbEverythingLib
## [everything-plugin](everything-plugin)
Rust binding for [Everything](https://www.voidtools.com/)'s [plugin SDK](https://www.voidtools.com/forum/viewtopic.php?t=16535).

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
