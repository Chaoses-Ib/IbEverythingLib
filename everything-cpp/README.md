# IbEverythingLib
A C++17 implementation of [Everything](https://www.voidtools.com/)'s (IPC) SDK.

## Features
- Higher performance. Compared with [the official SDK](https://www.voidtools.com/support/everything/sdk/), it reduces the query time by about 30%.
- Better asynchronous. Its sending blocking time is only 40% of the SDK. And it is based on [`std::future`](https://en.cppreference.com/w/cpp/thread/future.html), which gives you more features about asynchronous.
- Support [named instances](https://www.voidtools.com/en-us/support/everything/multiple_instances/#named_instances).
- Header-only and does not depend on the official DLL.

## Building
CMake:
```cmd
mkdir build
cd build
cmake ..
cmake --build . --config Release
```
For the test:
```
vcpkg install boost-test fmt
```
And add `-DBUILD_TESTING=ON` when calling `cmake ..` .