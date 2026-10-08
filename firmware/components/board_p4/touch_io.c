/* SPDX-License-Identifier: Apache-2.0 */
#include "touch_io.h"

#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include "esp_lcd_panel_io_interface.h"
#include "esp_timer.h"

#define GT911_STATUS_REG 0x814e
#define GT911_MAX_PARAM 40u
#define GT911_TRANSFER_WAIT_MS 50
#define GT911_FRAME_BUDGET_US 150000

typedef struct {
    esp_lcd_panel_io_t base;
    i2c_master_dev_handle_t device;
    esp_err_t first_error;
    bool status_valid;
    bool status_ready;
    bool ack_policy_valid;
    bool frame_active;
    int64_t frame_deadline_us;
} gt911_io_t;

_Static_assert(offsetof(gt911_io_t, base) == 0, "panel IO must be the first member");

static esp_err_t touch_rx(esp_lcd_panel_io_t *base, int command, void *params, size_t size);

static gt911_io_t *checked_io(esp_lcd_panel_io_handle_t base)
{
    return base && base->rx_param == touch_rx ? (gt911_io_t *)base : NULL;
}

static esp_err_t remember_error(gt911_io_t *io, esp_err_t error)
{
    if (error != ESP_OK) {
        if (io->first_error == ESP_OK) io->first_error = error;
        // A failed transaction cannot authorize skipping a later status ACK.
        io->ack_policy_valid = false;
        io->status_valid = false;
    }
    return error;
}

static int transfer_wait_ms(const gt911_io_t *io)
{
    if (!io->frame_active) return GT911_TRANSFER_WAIT_MS;
    const int64_t remaining_us = io->frame_deadline_us - esp_timer_get_time();
    if (remaining_us <= 0) return 0;
    const int64_t remaining_ms = remaining_us / 1000;
    return remaining_ms < GT911_TRANSFER_WAIT_MS ? (int)remaining_ms : GT911_TRANSFER_WAIT_MS;
}

static esp_err_t touch_rx(esp_lcd_panel_io_t *base, int command, void *params, size_t size)
{
    gt911_io_t *io = (gt911_io_t *)base;
    if (command < 0 || command > UINT16_MAX || !params || size == 0 || size > GT911_MAX_PARAM) {
        return remember_error(io, ESP_ERR_INVALID_ARG);
    }
    const bool status_read = command == GT911_STATUS_REG && size == 1;
    if (status_read) {
        io->status_valid = false;
        io->ack_policy_valid = false;
        io->frame_active = true;
        io->frame_deadline_us = esp_timer_get_time() + GT911_FRAME_BUDGET_US;
    }
    const int wait_ms = transfer_wait_ms(io);
    if (wait_ms <= 0) return remember_error(io, ESP_ERR_TIMEOUT);
    // GT911 registers are big endian even though coordinates are little endian.
    const uint8_t register_bytes[2] = {(uint8_t)(command >> 8), (uint8_t)command};
    const esp_err_t error = i2c_master_transmit_receive(io->device, register_bytes,
        sizeof(register_bytes), params, size, wait_ms);
    if (status_read && error == ESP_OK) {
        io->status_valid = true;
        io->status_ready = (*(const uint8_t *)params & 0x80) != 0;
        io->ack_policy_valid = true;
    }
    return remember_error(io, error);
}

static esp_err_t touch_tx(esp_lcd_panel_io_t *base, int command, const void *params, size_t size)
{
    gt911_io_t *io = (gt911_io_t *)base;
    if (command < 0 || command > UINT16_MAX || size > GT911_MAX_PARAM || (size && !params)) {
        return remember_error(io, ESP_ERR_INVALID_ARG);
    }
    const bool status_ack = command == GT911_STATUS_REG && size == 1
        && *(const uint8_t *)params == 0;
    // Upstream ACKs even when its preceding status read said "not ready".
    // A release frame can arrive between that read and write: sending zero
    // would discard the new frame and leave the driver's old contact active.
    // Suppress only this known no-frame ACK, never an ACK of a ready frame.
    if (status_ack && io->ack_policy_valid && !io->status_ready) {
        io->ack_policy_valid = false;
        io->frame_active = false;
        return ESP_OK;
    }
    const int wait_ms = transfer_wait_ms(io);
    if (wait_ms <= 0) return remember_error(io, ESP_ERR_TIMEOUT);
    uint8_t packet[GT911_MAX_PARAM + 2];
    packet[0] = (uint8_t)(command >> 8);
    packet[1] = (uint8_t)command;
    if (size) memcpy(packet + 2, params, size);
    const esp_err_t error = i2c_master_transmit(io->device, packet, size + 2, wait_ms);
    if (status_ack) {
        io->ack_policy_valid = false;
        io->frame_active = false;
    }
    return remember_error(io, error);
}

static esp_err_t unsupported_color(esp_lcd_panel_io_t *io, int command, const void *data, size_t size)
{
    (void)io; (void)command; (void)data; (void)size;
    return ESP_ERR_NOT_SUPPORTED;
}

static esp_err_t unsupported_callbacks(esp_lcd_panel_io_t *io,
    const esp_lcd_panel_io_callbacks_t *callbacks, void *context)
{
    (void)io; (void)callbacks; (void)context;
    return ESP_ERR_NOT_SUPPORTED;
}

static esp_err_t touch_delete(esp_lcd_panel_io_t *base)
{
    gt911_io_t *io = (gt911_io_t *)base;
    const esp_err_t error = i2c_master_bus_rm_device(io->device);
    if (error == ESP_OK) free(io);
    return error;
}

esp_err_t board_p4_new_gt911_io(i2c_master_bus_handle_t bus, uint8_t address,
                              esp_lcd_panel_io_handle_t *out_io)
{
    if (out_io) *out_io = NULL;
    if (!bus || !out_io || (address != 0x5d && address != 0x14)) return ESP_ERR_INVALID_ARG;
    gt911_io_t *io = calloc(1, sizeof(*io));
    if (!io) return ESP_ERR_NO_MEM;
    const i2c_device_config_t config = {
        .dev_addr_length = I2C_ADDR_BIT_LEN_7,
        .device_address = address,
        .scl_speed_hz = 400000,
    };
    const esp_err_t error = i2c_master_bus_add_device(bus, &config, &io->device);
    if (error != ESP_OK) {
        free(io);
        return error;
    }
    io->base.rx_param = touch_rx;
    io->base.tx_param = touch_tx;
    io->base.tx_color = unsupported_color;
    io->base.del = touch_delete;
    io->base.register_event_callbacks = unsupported_callbacks;
    *out_io = &io->base;
    return ESP_OK;
}

esp_err_t board_p4_gt911_io_take_error(esp_lcd_panel_io_handle_t base)
{
    gt911_io_t *io = checked_io(base);
    if (!io) return ESP_ERR_INVALID_ARG;
    const esp_err_t error = io->first_error;
    io->first_error = ESP_OK;
    return error;
}

bool board_p4_gt911_io_status_ready(esp_lcd_panel_io_handle_t base, bool *ready)
{
    gt911_io_t *io = checked_io(base);
    if (!io || !ready || !io->status_valid) return false;
    *ready = io->status_ready;
    return true;
}
