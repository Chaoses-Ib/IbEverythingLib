cmake_minimum_required(VERSION 3.14)

project(IbEverything)

include(FetchContent)
FetchContent_Declare(
    IbWinCpp
    GIT_REPOSITORY https://github.com/Chaoses-Ib/IbWinCppLib.git
    GIT_TAG        a29ac95537f403ac5ae221cb744d3e82076efbf7
)
FetchContent_MakeAvailable(IbWinCpp)

add_library(IbEverything source/src.cpp)
target_include_directories(IbEverything PUBLIC include)
target_compile_features(IbEverything PUBLIC cxx_std_17)
target_link_libraries(IbEverything PUBLIC IbWinCpp)

# test
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
    option(BUILD_TESTING "Build the testing tree." OFF)
    include(CTest)
    if(BUILD_TESTING)
        find_package(Boost REQUIRED COMPONENTS unit_test_framework)
        find_package(fmt CONFIG REQUIRED)

        add_executable(IbEverything_test test/test.cpp)
        target_link_libraries(IbEverything_test
            PRIVATE IbEverything
            PRIVATE Boost::unit_test_framework
            PRIVATE fmt::fmt
        )

        add_executable(IbEverything_time test/time.cpp)
        target_link_libraries(IbEverything_time PRIVATE IbEverything)
    endif()
endif()