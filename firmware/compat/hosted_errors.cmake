# The pinned upstream API aborts if the co-processor does not respond.
# Generate a local copy that returns that error to the settings service.
idf_component_get_property(hosted_dir espressif__esp_hosted COMPONENT_DIR)
file(READ "${hosted_dir}/host/api/src/esp_hosted_api.c" hosted_api)
string(FIND "${hosted_api}" "ESP_ERROR_CHECK(esp_hosted_reconfigure());" check_pos)
if(check_pos EQUAL -1)
    message(FATAL_ERROR "ESP-Hosted API changed; review the error-handling compatibility patch")
endif()
string(REPLACE "ESP_ERROR_CHECK(esp_hosted_reconfigure());"
    "do { esp_err_t e = esp_hosted_reconfigure(); if (e != ESP_OK) return e; } while (0);"
    hosted_api "${hosted_api}")
string(REPLACE "ESP_ERROR_CHECK(transport_drv_reconfigure());"
    "do { esp_err_t e = transport_drv_reconfigure(); if (e != ESP_OK) return e; } while (0);"
    hosted_api "${hosted_api}")
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/p4_hosted_api.c" "${hosted_api}")
target_sources(${hosted_lib} PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/p4_hosted_api.c")
