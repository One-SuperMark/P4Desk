#pragma once

#include <stdbool.h>
#include <stdint.h>
#include "esp_err.h"
#include "esp_lcd_panel_ops.h"
#include "esp_lcd_touch.h"

#define P4DESK_WIDTH 1024
#define P4DESK_HEIGHT 600
// Current enclosure placement is 180 degrees from the original bench setup.
// LCD and GT911 have opposite native axes; update output and input together.
#define P4DESK_DISPLAY_ROTATION_DEGREES 0
#define P4DESK_TOUCH_ROTATION_DEGREES 180
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
/**
 * Read one GT911 poll, including errors ignored by the managed driver's ACK
 * path. sample_ready is true only for a newly ready, successfully read frame;
 * old cached zero contacts on a not-ready poll are not a release indication.
 */
esp_err_t board_p4_touch_read(board_p4_t *board, bool *sample_ready);
esp_err_t board_p4_brightness(uint8_t percent);
uint64_t board_p4_sd_free_bytes(void);
// Call from one task at the UI performance cadence; internally limited to 30s.
// Logs read-only DMA underrun counters even when no UI frames were produced.
void board_p4_log_display_diagnostics(void);

// Read-only battery monitoring; charging remains under the board's hardware IC.
esp_err_t board_p4_battery_init(void);
int32_t board_p4_battery_voltage_mv(void);
