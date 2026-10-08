/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdbool.h>
#include <stdint.h>
#include "driver/i2c_master.h"
#include "esp_lcd_panel_io.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * GT911-only synchronous I2C transport, using the existing bus. Transactions
 * have finite waits; it never resets the shared bus or a GPIO. One touch task
 * owns both the driver and these accessors.
 */
esp_err_t board_p4_new_gt911_io(i2c_master_bus_handle_t bus, uint8_t address,
                              esp_lcd_panel_io_handle_t *out_io);

/**
 * Read and clear the first I/O error since the last call. The managed GT911
 * driver discards ACK errors in its zero-contact branch, so the touch task
 * must call this after read_data even when that call reports ESP_OK.
 */
esp_err_t board_p4_gt911_io_take_error(esp_lcd_panel_io_handle_t io);

/**
 * Last successful one-byte status read; false means no valid status exists.
 * Any later I/O error invalidates it, so cached points are never called fresh.
 */
bool board_p4_gt911_io_status_ready(esp_lcd_panel_io_handle_t io, bool *ready);

#ifdef __cplusplus
}
#endif
