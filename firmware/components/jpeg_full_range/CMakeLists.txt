# SPDX-License-Identifier: MIT
# The implementation stays in a generated copy of the original JPEG decoder.
# This component does not add a second decoder or a new application API.
idf_component_register(REQUIRES esp_driver_jpeg
                       REQUIRED_IDF_TARGETS esp32p4)

if(CMAKE_BUILD_EARLY_EXPANSION)
    return()
endif()

if(NOT IDF_VERSION_MAJOR EQUAL 6 OR NOT IDF_VERSION_MINOR EQUAL 0 OR
   NOT IDF_VERSION_PATCH EQUAL 2)
    message(FATAL_ERROR "jpeg_full_range requires the pinned ESP-IDF 6.0.2")
endif()
if(NOT CONFIG_SOC_JPEG_DECODE_SUPPORTED OR NOT CONFIG_SOC_DMA2D_SUPPORTED)
    message(FATAL_ERROR "jpeg_full_range requires the P4 JPEG decoder and 2D-DMA")
endif()

idf_component_get_property(p4desk_jpeg_dir esp_driver_jpeg COMPONENT_DIR)
idf_component_get_property(p4desk_dma_dir esp_driver_dma COMPONENT_DIR)
idf_build_get_property(p4desk_python PYTHON)
set(p4desk_jpeg_original "${p4desk_jpeg_dir}/jpeg_decode.c")
set(p4desk_jpeg_dma_source "${p4desk_dma_dir}/src/dma2d.c")
set(p4desk_jpeg_generated "${CMAKE_CURRENT_BINARY_DIR}/generated/jpeg_decode.c")
set(p4desk_jpeg_manifest "${CMAKE_CURRENT_BINARY_DIR}/generated/patch-manifest.json")
set(p4desk_jpeg_patch_tool "${CMAKE_CURRENT_LIST_DIR}/tools/generate_patch.py")

set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
             "${p4desk_jpeg_original}" "${p4desk_jpeg_dma_source}" "${p4desk_jpeg_patch_tool}")
execute_process(COMMAND "${p4desk_python}" "${p4desk_jpeg_patch_tool}"
                        --input "${p4desk_jpeg_original}"
                        --dma-source "${p4desk_jpeg_dma_source}"
                        --output "${p4desk_jpeg_generated}"
                        --manifest "${p4desk_jpeg_manifest}"
                RESULT_VARIABLE p4desk_jpeg_patch_result
                OUTPUT_VARIABLE p4desk_jpeg_patch_output
                ERROR_VARIABLE p4desk_jpeg_patch_error)
if(NOT p4desk_jpeg_patch_result EQUAL 0)
    message(FATAL_ERROR "JPEG full-range patch rejected: ${p4desk_jpeg_patch_error}")
endif()

include("${CMAKE_CURRENT_LIST_DIR}/cmake/replace_jpeg_source.cmake")
set_property(GLOBAL PROPERTY P4DESK_JPEG_FULL_RANGE_ORIGINAL "${p4desk_jpeg_original}")
set_property(GLOBAL PROPERTY P4DESK_JPEG_FULL_RANGE_GENERATED "${p4desk_jpeg_generated}")
cmake_language(DEFER DIRECTORY "${CMAKE_SOURCE_DIR}" CALL p4desk_jpeg_full_range_finish)
