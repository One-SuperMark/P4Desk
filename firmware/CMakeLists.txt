cmake_minimum_required(VERSION 3.22)

# C and Rust share the same ESP-IDF SDK, sdkconfig and C ABI.
set(IDF_TARGET esp32p4 CACHE STRING "P4Desk target")
set(COMPONENTS main rust_main)
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(p4desk VERSION 0.1.0)

if(NOT IDF_VERSION_MAJOR EQUAL 6 OR NOT IDF_VERSION_MINOR EQUAL 0 OR NOT IDF_VERSION_PATCH EQUAL 2)
    message(FATAL_ERROR "P4Desk requires ESP-IDF 6.0.2; activate the project's build environment")
endif()

# ESP-Hosted 1.4.7 predates IDF 6's split driver dependencies.
# Keep the registry source intact and declare the required driver links here.
idf_component_get_property(hosted_lib espressif__esp_hosted COMPONENT_LIB)
target_link_libraries(${hosted_lib} PRIVATE idf::esp_driver_sdmmc idf::sdmmc
    idf::esp_driver_gpio idf::esp_driver_spi idf::esp_driver_uart idf::esp_mm)

# Replace only the SDIO port wrapper: the IDF 6 controller must be initialized
# once, and a C6 error must never deinitialize the TF card's slot 0.
get_target_property(hosted_sources ${hosted_lib} SOURCES)
list(FILTER hosted_sources EXCLUDE REGEX "host/port/src/sdio_wrapper\\.c$")
list(FILTER hosted_sources EXCLUDE REGEX "host/esp_hosted_host_init\\.c$")
list(FILTER hosted_sources EXCLUDE REGEX "host/api/src/esp_hosted_api\\.c$")
set_property(TARGET ${hosted_lib} PROPERTY SOURCES ${hosted_sources})
target_sources(${hosted_lib} PRIVATE "${CMAKE_CURRENT_LIST_DIR}/compat/esp_hosted_sdio.c")
target_include_directories(${hosted_lib} PRIVATE
    "$ENV{IDF_PATH}/components/esp_driver_sdmmc/legacy/src")
include("${CMAKE_CURRENT_LIST_DIR}/compat/hosted_errors.cmake")
