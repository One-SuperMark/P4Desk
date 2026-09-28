cmake_minimum_required(VERSION 3.22)

# C and Rust share the same ESP-IDF SDK, sdkconfig and C ABI.
set(IDF_TARGET esp32p4 CACHE STRING "P4Desk target")
set(COMPONENTS main rust_main)
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(p4desk VERSION 0.1.0)

if(NOT IDF_VERSION_MAJOR EQUAL 6 OR NOT IDF_VERSION_MINOR EQUAL 0 OR NOT IDF_VERSION_PATCH EQUAL 2)
    message(FATAL_ERROR "P4Desk requires ESP-IDF 6.0.2; activate the project's build environment")
endif()
