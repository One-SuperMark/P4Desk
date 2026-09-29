#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef struct {
    uint16_t x, y;
    uint8_t id;
    uint8_t reserved[3];
} p4desk_touch_point_t;
typedef struct {
    uint64_t stamp_us;
    uint32_t session;
    uint16_t sequence;
    uint8_t count, reserved;
    p4desk_touch_point_t points[5];
} p4desk_touch_frame_t;
#ifdef __cplusplus
static_assert(sizeof(p4desk_touch_point_t) == 8, "touch point ABI");
static_assert(sizeof(p4desk_touch_frame_t) == 56, "touch frame ABI");
static_assert(offsetof(p4desk_touch_frame_t, points) == 16, "touch frame layout");
#else
_Static_assert(sizeof(p4desk_touch_point_t) == 8, "touch point ABI");
_Static_assert(sizeof(p4desk_touch_frame_t) == 56, "touch frame ABI");
_Static_assert(offsetof(p4desk_touch_frame_t, points) == 16, "touch frame layout");
#endif
#ifdef __cplusplus
extern "C" {
#endif
void rust_main_entry(void);
void host_lcd_draw_bitmap(int32_t x1, int32_t y1, int32_t x2, int32_t y2, const uint16_t *pixels);
bool host_touch_get_point(int32_t *x, int32_t *y);
bool p4desk_get_raw_touch(p4desk_touch_frame_t *frame);
void host_lcd_set_power(bool on);
void p4desk_pad_frame_begin(void);
void p4desk_pad_frame_end(void);
uint32_t p4desk_get_mode(void);
bool p4desk_set_mode(uint32_t mode, uint32_t session);
uint32_t p4desk_direct_jpeg_rotation_degrees(void);
bool p4desk_set_mode_with_jpeg_rotation(uint32_t mode, uint32_t session, uint32_t jpeg_rotation_degrees);
void p4desk_set_brightness(uint8_t percent);
int64_t p4desk_monotonic_us(void);
void p4desk_delay_ms(uint32_t milliseconds);
bool p4desk_sd_ready(void);
uint64_t p4desk_sd_free_bytes(void);
size_t p4desk_poll_packet(uint8_t *kind, uint16_t *sequence, uint8_t *buffer, size_t capacity);
bool p4desk_send_control(const char *json, size_t length, uint16_t sequence);
void p4desk_send_media(uint16_t usage);
bool p4desk_usb_connected(void);
void p4desk_heartbeat_received(void);
bool p4desk_host_active(void);
bool p4desk_ui_invalidated(void);
void p4desk_time_set(int64_t unix_ms);
int64_t p4desk_unix_ms(void);
#ifdef __cplusplus
}
#endif
