# IbEverythingLib
A C++17 library for [Everything](https://www.voidtools.com/).

## Features
* Higher performance. Compared with [the official SDK](https://www.voidtools.com/support/everything/sdk/), it reduces the query time by about 30%.
* Better asynchronous. Its sending blocking time is only 40% of the SDK. And it is based on std::future, which gives you more features about asynchronous.
* Support [named instances](https://www.voidtools.com/en-us/support/everything/multiple_instances/#named_instances).

## Building
```cmd
mkdir external
cd external
git clone --depth 1 https://github.com/Chaoses-Ib/IbWinCppLib.git
cd ..

mkdir build
cd build
cmake ..
cmake --build .
```