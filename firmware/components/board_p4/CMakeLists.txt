idf_component_register(SRCS "board_p4.c"
    INCLUDE_DIRS "include"
    REQUIRES esp_lcd esp_driver_i2c esp_driver_ledc esp_driver_gpio
             esp_lcd_touch esp_lcd_touch_gt911 esp_lcd_ek79007 fatfs sdmmc
    PRIV_REQUIRES esp_timer esp_hw_support esp_driver_sdmmc)
