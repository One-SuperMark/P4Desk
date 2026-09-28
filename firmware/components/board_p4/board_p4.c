/*
 * SPDX-FileCopyrightText: 2024 Espressif Systems (Shanghai) CO LTD
 * SPDX-License-Identifier: Apache-2.0
 *
 * Minimal hardware initialization adapted from Waveshare's
 * esp32_p4_wifi6_touch_lcd_7b BSP 3.0.1. The pin map, DSI power and panel
 * timing are retained; no graphics framework is initialized here.
 */
#include "board_p4.h"

#include <string.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdio.h>
#include <unistd.h>
#include "driver/i2c_master.h"
#include "driver/ledc.h"
#include "driver/sdmmc_host.h"
#include "esp_check.h"
#include "esp_ldo_regulator.h"
#include "esp_lcd_ek79007.h"
#include "esp_lcd_mipi_dsi.h"
#include "esp_lcd_touch_gt911.h"
#include "esp_log.h"
#include "esp_random.h"
#include "esp_vfs_fat.h"
#include "ff.h"
#include "sd_pwr_ctrl_by_on_chip_ldo.h"
#include "sdmmc_cmd.h"

static const char *TAG = "board_p4";
static esp_ldo_channel_handle_t s_dsi_power;
static i2c_master_bus_handle_t s_i2c;
static sdmmc_card_t *s_card;

static esp_err_t backlight_init(void)
{
    const ledc_timer_config_t timer = {
        .speed_mode = LEDC_LOW_SPEED_MODE,
        .duty_resolution = LEDC_TIMER_10_BIT,
        .timer_num = LEDC_TIMER_1,
        .freq_hz = 5000,
        .clk_cfg = LEDC_AUTO_CLK,
    };
    const ledc_channel_config_t channel = {
        .gpio_num = GPIO_NUM_32,
        .speed_mode = LEDC_LOW_SPEED_MODE,
        .channel = LEDC_CHANNEL_0,
        .intr_type = LEDC_INTR_DISABLE,
        .timer_sel = LEDC_TIMER_1,
        .duty = 0,
        .hpoint = 0,
        .flags.output_invert = 1,
    };
    ESP_RETURN_ON_ERROR(ledc_timer_config(&timer), TAG, "backlight timer");
    return ledc_channel_config(&channel);
}

esp_err_t board_p4_brightness(uint8_t percent)
{
    if (percent > 100) percent = 100;
    ESP_RETURN_ON_ERROR(ledc_set_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0,
                                    1023U * percent / 100U), TAG, "backlight duty");
    return ledc_update_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0);
}

static esp_err_t panel_init(board_p4_t *board)
{
    ESP_RETURN_ON_ERROR(backlight_init(), TAG, "backlight init");
    const esp_ldo_channel_config_t power = {.chan_id = 3, .voltage_mv = 2500};
    ESP_RETURN_ON_ERROR(esp_ldo_acquire_channel(&power, &s_dsi_power), TAG, "DSI power");

    esp_lcd_dsi_bus_handle_t bus = NULL;
    const esp_lcd_dsi_bus_config_t bus_config = {
        .bus_id = 0, .num_data_lanes = 2, .phy_clk_src = 0, .lane_bit_rate_mbps = 1000,
    };
    ESP_RETURN_ON_ERROR(esp_lcd_new_dsi_bus(&bus_config, &bus), TAG, "DSI bus");
    esp_lcd_panel_io_handle_t io = NULL;
    const esp_lcd_dbi_io_config_t dbi = {
        .virtual_channel = 0, .lcd_cmd_bits = 8, .lcd_param_bits = 8,
    };
    ESP_RETURN_ON_ERROR(esp_lcd_new_panel_io_dbi(bus, &dbi, &io), TAG, "panel IO");

    esp_lcd_dpi_panel_config_t dpi = EK79007_1024_600_PANEL_60HZ_CONFIG_CF(LCD_COLOR_FMT_RGB565);
    dpi.num_fbs = P4DESK_FB_COUNT;
    ek79007_vendor_config_t vendor = {
        .mipi_config = {.dsi_bus = bus, .dpi_config = &dpi},
    };
    const esp_lcd_panel_dev_config_t panel = {
        .reset_gpio_num = GPIO_NUM_33,
        .rgb_ele_order = LCD_RGB_ELEMENT_ORDER_RGB,
        .bits_per_pixel = 16,
        .vendor_config = &vendor,
    };
    ESP_RETURN_ON_ERROR(esp_lcd_new_panel_ek79007(io, &panel, &board->panel), TAG, "EK79007");
    ESP_RETURN_ON_ERROR(esp_lcd_panel_reset(board->panel), TAG, "panel reset");
    ESP_RETURN_ON_ERROR(esp_lcd_panel_init(board->panel), TAG, "panel init");
    void *framebuffers[P4DESK_FB_COUNT] = {0};
    ESP_RETURN_ON_ERROR(esp_lcd_dpi_panel_get_frame_buffer(board->panel, P4DESK_FB_COUNT,
        &framebuffers[0], &framebuffers[1], &framebuffers[2]), TAG, "framebuffers");
    for (unsigned n = 0; n < P4DESK_FB_COUNT; n++) board->framebuffers[n] = framebuffers[n];
    return ESP_OK;
}

static esp_err_t touch_init(board_p4_t *board)
{
    const i2c_master_bus_config_t i2c = {
        .i2c_port = I2C_NUM_0, .sda_io_num = GPIO_NUM_7, .scl_io_num = GPIO_NUM_8,
        .clk_source = I2C_CLK_SRC_DEFAULT, .glitch_ignore_cnt = 7,
        .flags.enable_internal_pullup = true,
    };
    ESP_RETURN_ON_ERROR(i2c_new_master_bus(&i2c, &s_i2c), TAG, "I2C");
    uint8_t addr = ESP_LCD_TOUCH_IO_I2C_GT911_ADDRESS;
    if (i2c_master_probe(s_i2c, addr, 100) != ESP_OK) {
        addr = ESP_LCD_TOUCH_IO_I2C_GT911_ADDRESS_BACKUP;
        if (i2c_master_probe(s_i2c, addr, 100) != ESP_OK) return ESP_ERR_NOT_FOUND;
    }
    esp_lcd_panel_io_i2c_config_t io_config = ESP_LCD_TOUCH_IO_I2C_GT911_CONFIG();
    io_config.dev_addr = addr;
    io_config.scl_speed_hz = 400000;
    esp_lcd_panel_io_handle_t io = NULL;
    ESP_RETURN_ON_ERROR(esp_lcd_new_panel_io_i2c(s_i2c, &io_config, &io), TAG, "touch IO");
    esp_lcd_touch_io_gt911_config_t driver_data = {.dev_addr = addr};
    const esp_lcd_touch_config_t config = {
        .x_max = P4DESK_WIDTH, .y_max = P4DESK_HEIGHT,
        .rst_gpio_num = GPIO_NUM_NC, .int_gpio_num = GPIO_NUM_NC,
        .levels = {.reset = 0, .interrupt = 0},
        // Calibrated orientation used by the working 7B painting firmware.
        .flags = {.swap_xy = false, .mirror_x = false, .mirror_y = false},
        .driver_data = &driver_data,
    };
    return esp_lcd_touch_new_i2c_gt911(io, &config, &board->touch);
}

static esp_err_t sd_mount(void)
{
    sdmmc_host_t host = SDMMC_HOST_DEFAULT();
    host.slot = SDMMC_HOST_SLOT_0;
    host.max_freq_khz = SDMMC_FREQ_HIGHSPEED;
    sd_pwr_ctrl_ldo_config_t power = {.ldo_chan_id = 4};
    sd_pwr_ctrl_handle_t power_handle = NULL;
    ESP_RETURN_ON_ERROR(sd_pwr_ctrl_new_on_chip_ldo(&power, &power_handle), TAG, "SD IO power");
    host.pwr_ctrl_handle = power_handle;
    const sdmmc_slot_config_t slot = {
        .cd = SDMMC_SLOT_NO_CD, .wp = SDMMC_SLOT_NO_WP, .width = 4, .flags = 0,
    };
    const esp_vfs_fat_sdmmc_mount_config_t mount = {
        .format_if_mount_failed = false, .max_files = 12, .allocation_unit_size = 64 * 1024,
    };
    // Slot 0 uses the 7B IO-MUX pins: CLK43/CMD44/D0..D3 39..42.
    return esp_vfs_fat_sdmmc_mount("/sdcard", &host, &slot, &mount, &s_card);
}

static esp_err_t sd_check(void)
{
    FATFS *filesystem = NULL;
    DWORD free_clusters = 0;
    FRESULT result = f_getfree("0:", &free_clusters, &filesystem);
    if (result != FR_OK || !filesystem) return ESP_FAIL;
    const char *format = filesystem->fs_type == FS_EXFAT ? "exFAT" :
        filesystem->fs_type == FS_FAT32 ? "FAT32" : filesystem->fs_type == FS_FAT16 ? "FAT16" : "FAT12";
    uint64_t total = 0, free_bytes = 0;
    ESP_RETURN_ON_ERROR(esp_vfs_fat_info("/sdcard", &total, &free_bytes), TAG, "TF capacity query");

    char path[80];
    int file = -1;
    for (unsigned attempt = 0; attempt < 3 && file < 0; attempt++) {
        snprintf(path, sizeof(path), "/sdcard/.p4desk-check-%08" PRIx32 "%08" PRIx32,
                 esp_random(), esp_random());
        file = open(path, O_RDWR | O_CREAT | O_EXCL, 0600);
    }
    if (file < 0) {
        ESP_LOGW(TAG, "TF format=%s total=%" PRIu64 " free=%" PRIu64 " read/write check unavailable", format, total, free_bytes);
        return ESP_FAIL;
    }
    static const uint8_t expected[16] = {0x50, 0x34, 0x44, 0x65, 0x73, 0x6b, 0x53, 0x44,
                                       0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef};
    uint8_t observed[sizeof(expected)] = {0};
    bool writable = write(file, expected, sizeof(expected)) == sizeof(expected) &&
        fsync(file) == 0 && lseek(file, 0, SEEK_SET) == 0 &&
        read(file, observed, sizeof(observed)) == sizeof(observed) &&
        memcmp(expected, observed, sizeof(expected)) == 0;
    if (close(file) != 0) writable = false;
    if (unlink(path) != 0) writable = false;
    ESP_LOGI(TAG, "TF format=%s total=%" PRIu64 " free=%" PRIu64 " read/write check=%s",
             format, total, free_bytes, writable ? "passed" : "failed");
    return writable ? ESP_OK : ESP_FAIL;
}

uint64_t board_p4_sd_free_bytes(void)
{
    if (!s_card) return 0;
    uint64_t total = 0, free_bytes = 0;
    if (esp_vfs_fat_info("/sdcard", &total, &free_bytes) != ESP_OK) return 0;
    return free_bytes;
}

esp_err_t board_p4_init(board_p4_t *board)
{
    memset(board, 0, sizeof(*board));
    ESP_RETURN_ON_ERROR(panel_init(board), TAG, "display initialization");
    esp_err_t err = touch_init(board);
    if (err != ESP_OK) ESP_LOGW(TAG, "touch unavailable: %s", esp_err_to_name(err));
    err = sd_mount();
    if (err == ESP_OK) err = sd_check();
    board->sd_ready = err == ESP_OK;
    if (err != ESP_OK) ESP_LOGW(TAG, "TF mount unavailable; card preserved: %s", esp_err_to_name(err));
    ESP_LOGI(TAG, "hardware ready: 1024x600 RGB565, touch=%d, TF=%d", board->touch != NULL, board->sd_ready);
    return ESP_OK;
}
