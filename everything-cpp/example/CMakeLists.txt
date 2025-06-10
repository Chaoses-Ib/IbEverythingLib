cmake_minimum_required(VERSION 3.2)

project(example)

add_subdirectory(.. IbEverything)

add_executable(example example.cpp)
target_link_libraries(example PRIVATE IbEverything)