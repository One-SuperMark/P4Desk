# The implementation is compiled in the patched esp_lcd DPI translation unit.
# This component publishes the header and schedules a project-local source swap.
idf_component_register(INCLUDE_DIRS "include"
                       REQUIRES esp_lcd
                       REQUIRED_IDF_TARGETS esp32p4)

if(CMAKE_BUILD_EARLY_EXPANSION)
    return()
endif()

if(NOT IDF_VERSION_MAJOR EQUAL 6 OR NOT IDF_VERSION_MINOR EQUAL 0 OR
   NOT IDF_VERSION_PATCH EQUAL 2)
    message(FATAL_ERROR "lcd_frame_observer requires the pinned ESP-IDF 6.0.2")
endif()
if(NOT CONFIG_SOC_MIPI_DSI_SUPPORTED OR NOT CONFIG_LCD_DSI_OBJ_FORCE_INTERNAL)
    message(FATAL_ERROR "lcd_frame_observer requires MIPI DSI and an internal-RAM DPI object")
endif()

idf_component_get_property(p4desk_lcd_dir esp_lcd COMPONENT_DIR)
idf_build_get_property(p4desk_python PYTHON)
set(p4desk_dpi_original "${p4desk_lcd_dir}/dsi/esp_lcd_panel_dpi.c")
set(p4desk_dpi_generated "${CMAKE_CURRENT_BINARY_DIR}/generated/esp_lcd_panel_dpi.c")
set(p4desk_dpi_manifest "${CMAKE_CURRENT_BINARY_DIR}/generated/patch-manifest.json")
set(p4desk_patch_tool "${CMAKE_CURRENT_LIST_DIR}/tools/generate_patch.py")

set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
             "${p4desk_dpi_original}" "${p4desk_patch_tool}"
             "${CMAKE_CURRENT_LIST_DIR}/include/p4desk_lcd_frame_observer.h")
execute_process(COMMAND "${p4desk_python}" "${p4desk_patch_tool}"
                        --input "${p4desk_dpi_original}"
                        --output "${p4desk_dpi_generated}"
                        --manifest "${p4desk_dpi_manifest}"
                RESULT_VARIABLE p4desk_patch_result
                OUTPUT_VARIABLE p4desk_patch_output
                ERROR_VARIABLE p4desk_patch_error)
if(NOT p4desk_patch_result EQUAL 0)
    message(FATAL_ERROR "DPI observer patch rejected: ${p4desk_patch_error}")
endif()

# esp_lcd can be registered after this component. Deferred arguments are read
# at execution time, so preserve paths in global properties instead of locals.
include("${CMAKE_CURRENT_LIST_DIR}/cmake/replace_dpi_source.cmake")
set_property(GLOBAL PROPERTY P4DESK_DPI_OBSERVER_ORIGINAL "${p4desk_dpi_original}")
set_property(GLOBAL PROPERTY P4DESK_DPI_OBSERVER_GENERATED "${p4desk_dpi_generated}")
set_property(GLOBAL PROPERTY P4DESK_DPI_OBSERVER_INCLUDE "${CMAKE_CURRENT_LIST_DIR}/include")
cmake_language(DEFER DIRECTORY "${CMAKE_SOURCE_DIR}" CALL p4desk_lcd_observer_finish)
