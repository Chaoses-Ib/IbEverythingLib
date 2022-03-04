cmake_minimum_required(VERSION 3.2)

project(IbEverything)

add_library(IbEverything source/src.cpp)
target_include_directories(IbEverything PUBLIC include)
target_compile_features(IbEverything PUBLIC cxx_std_17)
add_subdirectory(external/IbWinCppLib)
target_link_libraries(IbEverything PUBLIC IbWinCpp)

# tests
add_executable(test EXCLUDE_FROM_ALL test/test.cpp)
target_link_libraries(test PRIVATE IbEverything)
add_executable(test_time EXCLUDE_FROM_ALL test/time.cpp)
target_link_libraries(test_time PRIVATE IbEverything)