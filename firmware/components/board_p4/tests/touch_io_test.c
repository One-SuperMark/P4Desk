/* Compile the production IO against fake synchronous I2C operations. */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "touch_io.h"
#include "esp_lcd_panel_io_interface.h"

struct fake_bus { unsigned unused; };
struct fake_device { unsigned unused; };
static struct fake_bus bus;
static struct fake_device device;
static i2c_device_config_t added_config;
static int64_t now_us;
static esp_err_t add_error, remove_error, read_error, write_error;
static unsigned adds, removes, reads, writes;
static uint8_t next_status, last_packet[42];
static size_t last_size;
static int last_wait_ms;

int64_t esp_timer_get_time(void) { return now_us; }
esp_err_t i2c_master_bus_add_device(i2c_master_bus_handle_t input,
    const i2c_device_config_t *config, i2c_master_dev_handle_t *out)
{
    assert(input == &bus);
    ++adds;
    added_config = *config;
    if (add_error != ESP_OK) return add_error;
    *out = &device;
    return ESP_OK;
}
esp_err_t i2c_master_bus_rm_device(i2c_master_dev_handle_t input)
{
    assert(input == &device);
    ++removes;
    return remove_error;
}
static void record(i2c_master_dev_handle_t input, const uint8_t *bytes, size_t size, int wait_ms)
{
    assert(input == &device);
    assert(size <= sizeof(last_packet));
    assert(wait_ms > 0 && wait_ms <= 50);
    memcpy(last_packet, bytes, size);
    last_size = size;
    last_wait_ms = wait_ms;
}
esp_err_t i2c_master_transmit_receive(i2c_master_dev_handle_t input,
    const uint8_t *bytes, size_t size, uint8_t *output, size_t output_size, int wait_ms)
{
    record(input, bytes, size, wait_ms);
    ++reads;
    if (read_error != ESP_OK) return read_error;
    memset(output, 0, output_size);
    if (bytes[0] == 0x81 && bytes[1] == 0x4e && output_size == 1) output[0] = next_status;
    return ESP_OK;
}
esp_err_t i2c_master_transmit(i2c_master_dev_handle_t input,
    const uint8_t *bytes, size_t size, int wait_ms)
{
    record(input, bytes, size, wait_ms);
    ++writes;
    return write_error;
}
static esp_lcd_panel_io_handle_t create(void)
{
    esp_lcd_panel_io_handle_t io = NULL;
    assert(board_p4_new_gt911_io(&bus, 0x5d, &io) == ESP_OK && io);
    assert(added_config.device_address == 0x5d);
    assert(added_config.dev_addr_length == I2C_ADDR_BIT_LEN_7);
    assert(added_config.scl_speed_hz == 400000);
    assert(!added_config.flags.disable_ack_check);
    return io;
}
static void reset_fake(void)
{
    now_us = 0;
    add_error = remove_error = read_error = write_error = ESP_OK;
    adds = removes = reads = writes = 0;
    next_status = 0;
}
static void test_create_and_release(void)
{
    reset_fake();
    esp_lcd_panel_io_handle_t io = NULL;
    assert(board_p4_new_gt911_io(NULL, 0x5d, &io) == ESP_ERR_INVALID_ARG);
    assert(board_p4_new_gt911_io(&bus, 0x33, &io) == ESP_ERR_INVALID_ARG);
    assert(board_p4_new_gt911_io(&bus, 0x5d, NULL) == ESP_ERR_INVALID_ARG);
    assert(adds == 0);
    add_error = ESP_ERR_NO_MEM;
    assert(board_p4_new_gt911_io(&bus, 0x5d, &io) == ESP_ERR_NO_MEM && !io);
    add_error = ESP_OK;
    assert(board_p4_new_gt911_io(&bus, 0x14, &io) == ESP_OK);
    assert(added_config.device_address == 0x14);
    remove_error = ESP_FAIL;
    assert(io->del(io) == ESP_FAIL);
    remove_error = ESP_OK;
    assert(io->del(io) == ESP_OK);
    assert(removes == 2);
}
static void test_registers_and_ready_ack(void)
{
    reset_fake();
    esp_lcd_panel_io_handle_t io = create();
    uint8_t status = 0, coordinates[40], zero = 0;
    bool ready = false;
    assert(!board_p4_gt911_io_status_ready(io, &ready));
    next_status = 0x85;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK && status == 0x85);
    assert(last_size == 2 && last_packet[0] == 0x81 && last_packet[1] == 0x4e);
    assert(last_wait_ms == 50 && board_p4_gt911_io_status_ready(io, &ready) && ready);
    assert(io->rx_param(io, 0x814f, coordinates, sizeof(coordinates)) == ESP_OK);
    assert(last_packet[0] == 0x81 && last_packet[1] == 0x4f && last_wait_ms == 50);
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK);
    assert(writes == 1 && last_size == 3 && last_packet[2] == 0 && last_wait_ms == 50);
    // A zero-contact ready frame is still acknowledged, as is an ACK without a status read.
    next_status = 0x80;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK);
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK && writes == 2);
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK && writes == 3);
    assert(board_p4_gt911_io_take_error(io) == ESP_OK);
    assert(io->del(io) == ESP_OK);
}
static void test_no_frame_ack_preserves_new_release(void)
{
    reset_fake();
    esp_lcd_panel_io_handle_t io = create();
    uint8_t status, zero = 0;
    next_status = 0;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK);
    bool ready = true;
    assert(board_p4_gt911_io_status_ready(io, &ready) && !ready);
    // Simulate a ready release arriving after read but before the upstream no-frame ACK.
    next_status = 0x80;
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK && writes == 0);
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK && status == 0x80);
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK && writes == 1);
    assert(board_p4_gt911_io_status_ready(io, &ready) && ready);
    assert(board_p4_gt911_io_take_error(io) == ESP_OK);
    assert(io->del(io) == ESP_OK);
}
static void test_errors_are_sticky_and_no_false_suppression(void)
{
    reset_fake();
    esp_lcd_panel_io_handle_t io = create();
    uint8_t status, zero = 0, coordinates[8];
    bool ready = false;
    next_status = 0;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK);
    read_error = ESP_ERR_TIMEOUT;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_ERR_TIMEOUT);
    assert(!board_p4_gt911_io_status_ready(io, &ready));
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK && writes == 1);
    assert(board_p4_gt911_io_take_error(io) == ESP_ERR_TIMEOUT);
    assert(board_p4_gt911_io_take_error(io) == ESP_OK);
    read_error = ESP_OK;
    next_status = 0x80;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK);
    write_error = ESP_FAIL;
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_FAIL);
    assert(!board_p4_gt911_io_status_ready(io, &ready));
    // Even if upstream discards tx_param's return, the board accessor gets it.
    assert(board_p4_gt911_io_take_error(io) == ESP_FAIL);
    assert(board_p4_gt911_io_take_error(io) == ESP_OK);
    write_error = ESP_OK;
    next_status = 0x81;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK);
    read_error = ESP_FAIL;
    assert(io->rx_param(io, 0x814f, coordinates, sizeof(coordinates)) == ESP_FAIL);
    // GT911 cleared cached point_count before this failed coordinate read.
    // The status accessor must not mark that cached empty list as a release.
    assert(!board_p4_gt911_io_status_ready(io, &ready));
    read_error = ESP_OK;
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK);
    assert(board_p4_gt911_io_take_error(io) == ESP_FAIL);
    assert(board_p4_gt911_io_take_error(io) == ESP_OK);
    assert(io->del(io) == ESP_OK);
}
static void test_frame_deadline_and_input_bounds(void)
{
    reset_fake();
    esp_lcd_panel_io_handle_t io = create();
    uint8_t status, points[40], zero = 0;
    next_status = 0x81;
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK);
    now_us = 125000;
    assert(io->rx_param(io, 0x814f, points, sizeof(points)) == ESP_OK && last_wait_ms == 25);
    now_us = 150000;
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_ERR_TIMEOUT && writes == 0);
    assert(board_p4_gt911_io_take_error(io) == ESP_ERR_TIMEOUT);
    // A new status poll opens a new bounded budget and can recover normally.
    assert(io->rx_param(io, 0x814e, &status, 1) == ESP_OK && last_wait_ms == 50);
    assert(io->tx_param(io, 0x814e, &zero, 1) == ESP_OK);
    const unsigned call_count = reads + writes;
    assert(io->rx_param(io, -1, &status, 1) == ESP_ERR_INVALID_ARG);
    assert(io->rx_param(io, 65536, &status, 1) == ESP_ERR_INVALID_ARG);
    assert(io->rx_param(io, 0x814e, NULL, 1) == ESP_ERR_INVALID_ARG);
    assert(io->rx_param(io, 0x814e, &status, 0) == ESP_ERR_INVALID_ARG);
    assert(io->rx_param(io, 0x814f, points, 41) == ESP_ERR_INVALID_ARG);
    assert(io->tx_param(io, 0x814e, NULL, 1) == ESP_ERR_INVALID_ARG);
    assert(io->tx_param(io, 0x814e, points, 41) == ESP_ERR_INVALID_ARG);
    assert(reads + writes == call_count);
    assert(io->tx_color(io, 0, NULL, 0) == ESP_ERR_NOT_SUPPORTED);
    assert(io->register_event_callbacks(io, NULL, NULL) == ESP_ERR_NOT_SUPPORTED);
    assert(board_p4_gt911_io_take_error(io) == ESP_ERR_INVALID_ARG);
    assert(board_p4_gt911_io_take_error(NULL) == ESP_ERR_INVALID_ARG);
    assert(!board_p4_gt911_io_status_ready(io, NULL));
    assert(io->del(io) == ESP_OK);
}
int main(void)
{
    test_create_and_release();
    test_registers_and_ready_ack();
    test_no_frame_ack_preserves_new_release();
    test_errors_are_sticky_and_no_false_suppression();
    test_frame_deadline_and_input_bounds();
    puts("GT911 production IO tests: 5 cases passed");
    return 0;
}
