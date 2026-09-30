idf_component_register(SRCS "main.c" "p4desk_radio.c" "p4desk_runtime.c" "usb_device.c" "usb_rx_stream.c"
    "usb_tx_queue.c" "touch_exit_notice.c" "pad_touch_queue.c" "display_pixels.c" "display_ppa.c" "display_pipeline.c" "display_transition.c"
    "../../common/protocol/p4desk_protocol.c"
    INCLUDE_DIRS "." "tusb" "../../include" "../../common/protocol"
    PRIV_REQUIRES board_p4 rust_main esp_timer esp_driver_jpeg esp_driver_ppa esp_mm esp_lcd esp_lcd_touch lcd_frame_observer jpeg_full_range
                  esp_wifi esp_netif esp_event nvs_flash bt espressif__esp_hosted spiffs esp_partition esp_hw_support esp_driver_usb_serial_jtag espressif__tinyusb)

idf_component_get_property(tusb_lib espressif__tinyusb COMPONENT_LIB)
target_include_directories(${tusb_lib} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/tusb")
target_sources(${tusb_lib} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/tusb/usb_descriptors.c")
