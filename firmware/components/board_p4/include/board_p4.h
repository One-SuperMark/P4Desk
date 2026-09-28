#pragma once

#include <stdbool.h>
#include <stdint.h>
#include "esp_err.h"
#include "esp_lcd_panel_ops.h"
#include "esp_lcd_touch.h"

#define P4DESK_WIDTH 1024
#define P4DESK_HEIGHT 600
// Matches the working 7B paint firmware: rotate display pixels, keep GT911 raw.
// Panel and touch have different native axes, so calibrate each explicitly.
#define P4DESK_DISPLAY_ROTATION_DEGREES 180
#define P4DESK_TOUCH_ROTATION_DEGREES 0
#if (P4DESK_DISPLAY_ROTATION_DEGREES != 0 && P4DESK_DISPLAY_ROTATION_DEGREES != 180) || \
    (P4DESK_TOUCH_ROTATION_DEGREES != 0 && P4DESK_TOUCH_ROTATION_DEGREES != 180)
#error "P4Desk display and touch rotations must be 0 or 180"
#endif
#define P4DESK_FB_COUNT 3
#define P4DESK_FB_BYTES (P4DESK_WIDTH * P4DESK_HEIGHT * sizeof(uint16_t))

typedef struct {
    esp_lcd_panel_handle_t panel;
    esp_lcd_touch_handle_t touch;
    uint16_t *framebuffers[P4DESK_FB_COUNT];
    bool sd_ready;
} board_p4_t;

esp_err_t board_p4_init(board_p4_t *board);
esp_err_t board_p4_brightness(uint8_t percent);
uint64_t board_p4_sd_free_bytes(void);
