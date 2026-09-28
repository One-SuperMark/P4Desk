#include "p4desk_runtime.h"
#include "p4desk_hal.h"
#include "touch_exit_notice.h"

#include <inttypes.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include "driver/jpeg_decode.h"
#include "esp_check.h"
#include "esp_heap_caps.h"
#include "esp_lcd_mipi_dsi.h"
#include "esp_log.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/queue.h"
#include "freertos/semphr.h"
#include "freertos/task.h"

_Static_assert(sizeof(time_t) == 8, "Rust and C require the same ESP-IDF time64 ABI");
_Static_assert(sizeof(bool) == 1, "C bool must match Rust bool");
_Static_assert(sizeof(int32_t) == 4 && sizeof(uint32_t) == 4, "HAL 32-bit integer ABI");
_Static_assert(sizeof(int64_t) == 8 && sizeof(uint64_t) == 8, "HAL 64-bit integer ABI");
_Static_assert(sizeof(uint16_t) == 2, "HAL pixel and sequence ABI");
_Static_assert(sizeof(void *) == 4 && sizeof(size_t) == 4, "P4 Rust target uses 32-bit pointers");

#define MODE_PAD 0U
#define MODE_DISPLAY 1U
#define CONTROL_QUEUE_COUNT 8
#define JPEG_QUEUE_COUNT 2
#define HEARTBEAT_TIMEOUT_US 3000000LL
#define DIAGNOSTICS_INTERVAL_US 30000000LL

typedef struct {
    p4p_header_t header;
    uint8_t *payload;
    uint32_t epoch;
    uint32_t session;
} owned_packet_t;

typedef enum { FB_FREE, FB_BUILDING, FB_PENDING, FB_SCANNING, FB_RETIRING } framebuffer_state_t;
typedef struct {
    uint8_t active;
    framebuffer_state_t states[P4DESK_FB_COUNT];
} display_owner_t;

static const char *TAG = "display_owner";
static board_p4_t *s_board;
static QueueHandle_t s_control_queue, s_jpeg_queue;
static SemaphoreHandle_t s_pad_lock, s_refresh_event, s_brightness_lock;
static uint16_t *s_pad_pixels;
static _Atomic(TaskHandle_t) s_pad_writer;
static jpeg_decoder_handle_t s_decoder;
static uint8_t *s_decoded;
static size_t s_decoded_capacity;
static atomic_bool s_pad_dirty;
static atomic_uint s_refresh_count;
static atomic_uint s_presented_frames, s_bad_jpeg;
static portMUX_TYPE s_state_lock = portMUX_INITIALIZER_UNLOCKED;
static uint32_t s_mode = MODE_PAD, s_epoch = 1, s_session;
static bool s_connected, s_invalidated = true, s_time_valid;
static int64_t s_heartbeat_us;
static atomic_uchar s_brightness = 75;
static atomic_bool s_backlight_on = true;
static bool s_pad_touch_active, s_pad_touch_blocked;
static uint8_t s_pad_touch_id;
static int32_t s_pad_touch_x, s_pad_touch_y;
static p4desk_touch_frame_t s_raw_touch;
static bool s_raw_touch_valid;

static void state_snapshot(uint32_t *mode, uint32_t *epoch, uint32_t *session)
{
    portENTER_CRITICAL(&s_state_lock);
    if (mode) *mode = s_mode;
    if (epoch) *epoch = s_epoch;
    if (session) *session = s_session;
    portEXIT_CRITICAL(&s_state_lock);
}

uint32_t p4desk_epoch(void)
{
    uint32_t epoch;
    state_snapshot(NULL, &epoch, NULL);
    return epoch;
}

uint32_t p4desk_get_mode(void)
{
    uint32_t mode;
    state_snapshot(&mode, NULL, NULL);
    return mode;
}

static void discard_packets(QueueHandle_t queue)
{
    owned_packet_t packet;
    while (xQueueReceive(queue, &packet, 0) == pdTRUE) free(packet.payload);
}

bool p4desk_set_mode(uint32_t mode, uint32_t session)
{
    if (mode > MODE_DISPLAY) return false;
    bool changed = false;
    portENTER_CRITICAL(&s_state_lock);
    if (mode == MODE_DISPLAY && !s_connected) {
        portEXIT_CRITICAL(&s_state_lock);
        return false;
    }
    if (s_mode != mode || (mode == MODE_DISPLAY && s_session != session)) {
        s_mode = mode;
        s_session = session;
        ++s_epoch;
        s_invalidated = true;
        s_pad_touch_active = false;
        s_pad_touch_blocked = true; // Re-arm after all contacts are released.
        changed = true;
    }
    if (mode == MODE_DISPLAY) s_heartbeat_us = esp_timer_get_time();
    portEXIT_CRITICAL(&s_state_lock);
    if (changed) {
        discard_packets(s_jpeg_queue);
        // The USB owner must consume an in-flight message to its boundary.
        // Its original epoch discards stale JPEGs after this mode change.
        atomic_store(&s_pad_dirty, true);
    }
    return true;
}

bool p4desk_usb_connected(void)
{
    portENTER_CRITICAL(&s_state_lock);
    bool connected = s_connected;
    portEXIT_CRITICAL(&s_state_lock);
    return connected;
}

void p4desk_heartbeat_received(void)
{
    portENTER_CRITICAL(&s_state_lock);
    s_heartbeat_us = esp_timer_get_time();
    portEXIT_CRITICAL(&s_state_lock);
}

bool p4desk_host_active(void)
{
    int64_t now = esp_timer_get_time();
    portENTER_CRITICAL(&s_state_lock);
    bool active = s_connected && s_heartbeat_us > 0 && now - s_heartbeat_us < HEARTBEAT_TIMEOUT_US;
    portEXIT_CRITICAL(&s_state_lock);
    return active;
}

void p4desk_usb_mount_changed(bool connected)
{
    portENTER_CRITICAL(&s_state_lock);
    s_connected = connected;
    s_heartbeat_us = 0;
    s_invalidated = true;
    portEXIT_CRITICAL(&s_state_lock);
    if (!connected) {
        p4desk_set_mode(MODE_PAD, 0);
        discard_packets(s_control_queue);
        discard_packets(s_jpeg_queue);
    }
}

bool p4desk_ui_invalidated(void)
{
    portENTER_CRITICAL(&s_state_lock);
    bool invalidated = s_invalidated;
    s_invalidated = false;
    portEXIT_CRITICAL(&s_state_lock);
    return invalidated;
}

static void packet_rejected(const p4p_header_t *header, const char *error)
{
    if (header->kind == P4P_KIND_JPEG) return;
    char message[192];
    int length = snprintf(message, sizeof(message),
        "{\"op\":\"ack\",\"request_id\":%u,\"acknowledged\":\"%s\",\"ok\":false,\"error\":\"%s\"}",
        header->sequence, header->kind == P4P_KIND_RESOURCE ? "resource" : "control", error);
    if (length > 0 && length < sizeof(message)) p4desk_send_control(message, length, header->sequence);
}

void p4desk_receive_message(const p4p_header_t *header, const uint8_t *payload, uint32_t epoch)
{
    uint32_t mode, current_epoch, session;
    state_snapshot(&mode, &current_epoch, &session);
    if (header->kind == P4P_KIND_JPEG && (mode != MODE_DISPLAY || epoch != current_epoch)) return;
    owned_packet_t packet = {.header = *header, .epoch = epoch, .session = session};
    packet.payload = heap_caps_malloc(header->payload_length, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (!packet.payload) { packet_rejected(header, "out_of_memory"); return; }
    memcpy(packet.payload, payload, header->payload_length);
    QueueHandle_t queue = header->kind == P4P_KIND_JPEG ? s_jpeg_queue : s_control_queue;
    if (xQueueSend(queue, &packet, 0) == pdTRUE) return;
    if (header->kind == P4P_KIND_JPEG) {
        owned_packet_t stale;
        if (xQueueReceive(queue, &stale, 0) == pdTRUE) free(stale.payload);
        if (xQueueSend(queue, &packet, 0) == pdTRUE) return;
    }
    free(packet.payload);
    packet_rejected(header, "queue_full");
}

size_t p4desk_poll_packet(uint8_t *kind, uint16_t *sequence, uint8_t *buffer, size_t capacity)
{
    if (!kind || !sequence || !buffer) return 0;
    owned_packet_t packet;
    if (xQueueReceive(s_control_queue, &packet, 0) != pdTRUE) return 0;
    size_t length = packet.header.payload_length;
    if (length > capacity) {
        packet_rejected(&packet.header, "receive_buffer_too_small");
        free(packet.payload);
        return 0;
    }
    *kind = packet.header.kind;
    *sequence = packet.header.sequence;
    memcpy(buffer, packet.payload, length);
    free(packet.payload);
    return length;
}

bool p4desk_send_control(const char *json, size_t length, uint16_t sequence)
{
    return p4desk_usb_queue_json(json, length, sequence, true);
}

void p4desk_send_media(uint16_t usage) { p4desk_usb_queue_media(usage); }

void p4desk_pad_frame_begin(void)
{
    xSemaphoreTake(s_pad_lock, portMAX_DELAY);
    atomic_store(&s_pad_writer, xTaskGetCurrentTaskHandle());
}

void p4desk_pad_frame_end(void)
{
    if (atomic_load(&s_pad_writer) != xTaskGetCurrentTaskHandle()) return;
    atomic_store(&s_pad_writer, NULL);
    atomic_store(&s_pad_dirty, true);
    xSemaphoreGive(s_pad_lock);
}

void host_lcd_draw_bitmap(int32_t x1, int32_t y1, int32_t x2, int32_t y2, const uint16_t *pixels)
{
    if (!pixels || x2 <= x1 || y2 <= y1) return;
    bool own_lock = atomic_load(&s_pad_writer) != xTaskGetCurrentTaskHandle();
    if (own_lock) xSemaphoreTake(s_pad_lock, portMAX_DELAY);
    const int32_t stride = x2 - x1;
    int32_t left = x1 < 0 ? 0 : x1, top = y1 < 0 ? 0 : y1;
    int32_t right = x2 > P4DESK_WIDTH ? P4DESK_WIDTH : x2;
    int32_t bottom = y2 > P4DESK_HEIGHT ? P4DESK_HEIGHT : y2;
    if (left < right && top < bottom) {
        // The Rust flush pointer is borrowed only during this synchronous call.
        for (int32_t row = top; row < bottom; row++) {
            memcpy(s_pad_pixels + row * P4DESK_WIDTH + left,
                   pixels + (row - y1) * stride + left - x1,
                   (right - left) * sizeof(uint16_t));
        }
    }
    if (own_lock) {
        atomic_store(&s_pad_dirty, true);
        xSemaphoreGive(s_pad_lock);
    }
}

bool host_touch_get_point(int32_t *x, int32_t *y)
{
    if (!x || !y) return false;
    portENTER_CRITICAL(&s_state_lock);
    bool active = s_mode == MODE_PAD && s_pad_touch_active;
    *x = s_pad_touch_x;
    *y = s_pad_touch_y;
    portEXIT_CRITICAL(&s_state_lock);
    return active;
}

bool p4desk_get_raw_touch(p4desk_touch_frame_t *frame)
{
    if (!frame) return false;
    portENTER_CRITICAL(&s_state_lock);
    bool valid = s_raw_touch_valid;
    *frame = s_raw_touch;
    portEXIT_CRITICAL(&s_state_lock);
    return valid;
}

static void apply_backlight(void)
{
    xSemaphoreTake(s_brightness_lock, portMAX_DELAY);
    board_p4_brightness(atomic_load(&s_backlight_on) ? atomic_load(&s_brightness) : 0);
    xSemaphoreGive(s_brightness_lock);
}

void p4desk_set_brightness(uint8_t percent)
{
    atomic_store(&s_brightness, percent > 100 ? 100 : percent);
    apply_backlight();
}

void host_lcd_set_power(bool on)
{
    atomic_store(&s_backlight_on, on);
    apply_backlight();
}
int64_t p4desk_monotonic_us(void) { return esp_timer_get_time(); }
void p4desk_delay_ms(uint32_t milliseconds) { vTaskDelay(pdMS_TO_TICKS(milliseconds)); }
bool p4desk_sd_ready(void) { return s_board->sd_ready; }
uint64_t p4desk_sd_free_bytes(void) { return s_board->sd_ready ? board_p4_sd_free_bytes() : 0; }

void p4desk_time_set(int64_t unix_ms)
{
    // UTC 2000-01-01 through 2100-01-01, inclusive; shared with Rust validation.
    if (unix_ms < 946684800000LL || unix_ms > 4102444800000LL) return;
    struct timeval value = {.tv_sec = unix_ms / 1000, .tv_usec = (unix_ms % 1000) * 1000};
    if (settimeofday(&value, NULL) == 0) {
        portENTER_CRITICAL(&s_state_lock);
        s_time_valid = true;
        s_invalidated = true;
        portEXIT_CRITICAL(&s_state_lock);
    }
}

int64_t p4desk_unix_ms(void)
{
    portENTER_CRITICAL(&s_state_lock);
    bool valid = s_time_valid;
    portEXIT_CRITICAL(&s_state_lock);
    if (!valid) return 0;
    struct timeval value;
    gettimeofday(&value, NULL);
    return (int64_t)value.tv_sec * 1000 + value.tv_usec / 1000;
}

static bool refresh_done(esp_lcd_panel_handle_t panel, esp_lcd_dpi_panel_event_data_t *event, void *context)
{
    (void)panel; (void)event; (void)context;
    atomic_fetch_add_explicit(&s_refresh_count, 1, memory_order_relaxed);
    BaseType_t wake = pdFALSE;
    xSemaphoreGiveFromISR(s_refresh_event, &wake);
    return wake == pdTRUE;
}

static bool frame_current(const owned_packet_t *packet)
{
    uint32_t mode, epoch, session;
    state_snapshot(&mode, &epoch, &session);
    return mode == MODE_DISPLAY && epoch == packet->epoch && session == packet->session;
}

static bool jpeg_mcu_dimensions(const uint8_t *bytes, size_t length, uint32_t *width, uint32_t *height)
{
    if (length < 4 || bytes[0] != 0xff || bytes[1] != 0xd8) return false;
    size_t offset = 2;
    while (offset + 1 < length) {
        if (bytes[offset++] != 0xff) return false;
        while (offset < length && bytes[offset] == 0xff) offset++;
        if (offset == length) return false;
        uint8_t marker = bytes[offset++];
        if (marker == 0xd9 || marker == 0xda || marker == 0) return false;
        if (marker == 0xd8 || (marker >= 0xd0 && marker <= 0xd7)) continue;
        if (offset + 2 > length) return false;
        size_t segment = ((size_t)bytes[offset] << 8) | bytes[offset + 1];
        if (segment < 2 || segment > length - offset) return false;
        if (marker == 0xc0) {
            if (segment < 11 || bytes[offset + 2] != 8) return false;
            uint8_t components = bytes[offset + 7];
            if ((components != 1 && components != 3) || segment < (size_t)8 + components * 3) return false;
            uint8_t sampling = bytes[offset + 9];
            uint8_t horizontal = sampling >> 4, vertical = sampling & 0xf;
            if (!horizontal || !vertical || horizontal > 4 || vertical > 4) return false;
            *width = horizontal * 8;
            *height = vertical * 8;
            return true;
        }
        // Only baseline DCT is supported by the P4 hardware decoder.
        if ((marker >= 0xc1 && marker <= 0xc3) || (marker >= 0xc5 && marker <= 0xc7) ||
            (marker >= 0xc9 && marker <= 0xcb) || (marker >= 0xcd && marker <= 0xcf)) return false;
        offset += segment;
    }
    return false;
}

static bool reject_jpeg(void)
{
    atomic_fetch_add_explicit(&s_bad_jpeg, 1, memory_order_relaxed);
    return false;
}

static bool decode_frame(const owned_packet_t *packet, uint16_t *target)
{
    jpeg_decode_picture_info_t info;
    if (jpeg_decoder_get_info(packet->payload, packet->header.payload_length, &info) != ESP_OK ||
        info.width != P4DESK_WIDTH || info.height != P4DESK_HEIGHT) return reject_jpeg();
    uint32_t mcu_x, mcu_y;
    if (!jpeg_mcu_dimensions(packet->payload, packet->header.payload_length, &mcu_x, &mcu_y)) return reject_jpeg();
    if (info.sample_method != JPEG_DOWN_SAMPLING_YUV422 && info.sample_method != JPEG_DOWN_SAMPLING_YUV420 &&
        info.sample_method != JPEG_DOWN_SAMPLING_YUV444 && info.sample_method != JPEG_DOWN_SAMPLING_GRAY) return reject_jpeg();
    uint32_t stride = (info.width + mcu_x - 1) / mcu_x * mcu_x;
    uint32_t rows = (info.height + mcu_y - 1) / mcu_y * mcu_y;
    bool gray = info.sample_method == JPEG_DOWN_SAMPLING_GRAY;
    size_t required = (size_t)stride * rows * (gray ? 1 : 2);
    if (required > s_decoded_capacity) return reject_jpeg();
    jpeg_decode_cfg_t config = {
        .output_format = gray ? JPEG_DECODE_OUT_FORMAT_GRAY : JPEG_DECODE_OUT_FORMAT_RGB565,
        // Matches the official 7B app_lcd_p4.c RGB565 decoder setting. BGR is
        // the driver's little-endian output setting; native RGB565 words can
        // be copied to the RGB element-order DSI panel. Color blocks remain an
        // explicit hardware acceptance check.
        .rgb_order = JPEG_DEC_RGB_ELEMENT_ORDER_BGR, .conv_std = JPEG_YUV_RGB_CONV_STD_BT601,
    };
    uint32_t written = 0;
    if (jpeg_decoder_process(s_decoder, &config, packet->payload, packet->header.payload_length,
                            s_decoded, s_decoded_capacity, &written) != ESP_OK || written != required) return reject_jpeg();
    if (!frame_current(packet)) return false;
    for (uint32_t y = 0; y < P4DESK_HEIGHT; y++) {
        if (gray) {
            for (uint32_t x = 0; x < P4DESK_WIDTH; x++) {
                uint8_t v = s_decoded[y * stride + x];
                target[y * P4DESK_WIDTH + x] = ((uint16_t)(v >> 3) << 11) | ((uint16_t)(v >> 2) << 5) | (v >> 3);
            }
        } else {
            // Crop MCU padding (e.g. YUV420 decodes 600 visible rows to 608).
            memcpy(target + y * P4DESK_WIDTH, s_decoded + y * stride * 2, P4DESK_WIDTH * 2);
        }
    }
    return frame_current(packet);
}

static bool present_buffer(display_owner_t *owner, uint8_t index)
{
    if (owner->states[index] != FB_BUILDING || owner->states[owner->active] != FB_SCANNING) abort();
    uint8_t previous = owner->active;
    owner->states[index] = FB_PENDING;
    owner->states[previous] = FB_RETIRING;
    // In IDF 6.0.2 dpi_panel_draw_bitmap_2d recognizes its own framebuffers and
    // writes back the selected rows' cache before switching cur_fb_index.
    if (esp_lcd_panel_draw_bitmap(s_board->panel, 0, 0, P4DESK_WIDTH, P4DESK_HEIGHT,
                                  s_board->framebuffers[index]) != ESP_OK) return false;
    uint32_t submitted = atomic_load_explicit(&s_refresh_count, memory_order_relaxed);
    // IDF returns immediately when selecting one of its own framebuffers.
    // Two refresh callbacks after selection ensure the old DMA scan has ended
    // and the selected frame has completed a refresh before any buffer reuse.
    while ((uint32_t)(atomic_load_explicit(&s_refresh_count, memory_order_relaxed) - submitted) < 2) {
        if (xSemaphoreTake(s_refresh_event, pdMS_TO_TICKS(1000)) != pdTRUE) {
            ESP_LOGE(TAG, "DSI refresh deadline missed; buffers remain owned");
            return false;
        }
    }
    owner->states[previous] = FB_FREE;
    owner->states[index] = FB_SCANNING;
    owner->active = index;
    // Count actual successful LCD submissions (Pad and USB), after refresh.
    atomic_fetch_add_explicit(&s_presented_frames, 1, memory_order_relaxed);
    return true;
}

static void display_task(void *argument)
{
    (void)argument;
    display_owner_t owner = {.active = 0, .states = {FB_SCANNING, FB_FREE, FB_FREE}};
    bool first_frame = true;
    for (;;) {
        uint32_t mode, epoch;
        state_snapshot(&mode, &epoch, NULL);
        uint8_t building = (owner.active + 1) % P4DESK_FB_COUNT;
        if (owner.states[building] != FB_FREE) abort();
        if (mode == MODE_PAD) {
            bool dirty = atomic_exchange(&s_pad_dirty, false);
            if (!first_frame && !dirty) {
                vTaskDelay(pdMS_TO_TICKS(5));
                continue;
            }
            owner.states[building] = FB_BUILDING;
            xSemaphoreTake(s_pad_lock, portMAX_DELAY);
            memcpy(s_board->framebuffers[building], s_pad_pixels, P4DESK_FB_BYTES);
            xSemaphoreGive(s_pad_lock);
            uint32_t current_mode, current_epoch;
            state_snapshot(&current_mode, &current_epoch, NULL);
            if (current_mode != MODE_PAD || current_epoch != epoch) {
                owner.states[building] = FB_FREE;
                continue;
            }
            if (!present_buffer(&owner, building)) {
                // A missed refresh makes the DMA owner uncertain: do not write
                // to either buffer again. A reset is safer than visible damage.
                abort();
            }
            first_frame = false;
            apply_backlight();
        } else {
            owned_packet_t packet;
            if (xQueueReceive(s_jpeg_queue, &packet, pdMS_TO_TICKS(20)) != pdTRUE) continue;
            owner.states[building] = FB_BUILDING;
            if (frame_current(&packet) && decode_frame(&packet, s_board->framebuffers[building])) {
                if (!present_buffer(&owner, building)) abort();
                if (frame_current(&packet)) {
                    char message[192];
                    int length = snprintf(message, sizeof(message),
                        "{\"op\":\"frame_presented\",\"session\":%" PRIu32 ",\"sequence\":%u,\"device_us\":%" PRId64 "}",
                        packet.session, packet.header.sequence, esp_timer_get_time());
                    if (length > 0 && length < sizeof(message)) p4desk_send_control(message, length, packet.header.sequence);
                }
            } else {
                owner.states[building] = FB_FREE;
            }
            free(packet.payload);
        }
    }
}

static void send_touch(const esp_lcd_touch_point_data_t *points, uint8_t count, uint32_t session, uint16_t sequence)
{
    char json[512];
    int length = snprintf(json, sizeof(json),
        "{\"op\":\"touch\",\"session\":%" PRIu32 ",\"sequence\":%u,\"stamp_us\":%" PRId64 ",\"points\":[",
        session, sequence, esp_timer_get_time());
    if (length < 0 || length >= sizeof(json)) return;
    for (uint8_t n = 0; n < count; n++) {
        int added = snprintf(json + length, sizeof(json) - length,
            "%s{\"id\":%u,\"x\":%u,\"y\":%u}", n ? "," : "", points[n].track_id, points[n].x, points[n].y);
        if (added < 0 || added >= sizeof(json) - length) return;
        length += added;
    }
    if (length + 2 >= sizeof(json)) return;
    json[length++] = ']'; json[length++] = '}';
    // Touch JSON uses the full 16-bit sample sequence. Only the shared wire
    // header uses 10 bits; control request IDs remain unchanged.
    p4desk_usb_queue_touch(json, length, sequence & 1023, count == 0);
}

static bool send_exit_notice(void *context)
{
    (void)context;
    const char message[] = "{\"op\":\"request_mode\",\"mode\":\"pad\"}";
    return p4desk_send_control(message, sizeof(message) - 1, 0);
}

static void touch_task(void *argument)
{
    (void)argument;
    uint16_t sequence = 0;
    uint8_t old_count = 0;
    uint32_t old_session = 0;
    int64_t three_since = 0;
    uint16_t raw_sequence = 0;
    p4desk_exit_notice_t exit_notice = {0};
    for (;;) {
        esp_lcd_touch_point_data_t points[5] = {0};
        uint8_t count = 0;
        if (s_board->touch && esp_lcd_touch_read_data(s_board->touch) == ESP_OK)
            esp_lcd_touch_get_data(s_board->touch, points, &count, 5);
        if (count > 5) count = 5;
        for (uint8_t n = 0; n < count; n++) {
            if (points[n].x >= P4DESK_WIDTH) points[n].x = P4DESK_WIDTH - 1;
            if (points[n].y >= P4DESK_HEIGHT) points[n].y = P4DESK_HEIGHT - 1;
            // Match the LCD's 180-degree rotation before all input consumers.
            // Clamp first; the generic touch mirror uses width-x, not width-1-x.
            points[n].x = (P4DESK_WIDTH - 1) - points[n].x;
            points[n].y = (P4DESK_HEIGHT - 1) - points[n].y;
        }
        // Publish one coherent raw sample, including empty release frames.
        // Pad's latched primary contact below remains a separate UI policy.
        p4desk_touch_frame_t raw = {
            .stamp_us = esp_timer_get_time(), .sequence = raw_sequence++, .count = count,
        };
        for (uint8_t n = 0; n < count; n++) {
            raw.points[n].x = points[n].x;
            raw.points[n].y = points[n].y;
            raw.points[n].id = points[n].track_id;
        }
        portENTER_CRITICAL(&s_state_lock);
        raw.session = s_session;
        s_raw_touch = raw;
        s_raw_touch_valid = true;
        portEXIT_CRITICAL(&s_state_lock);
        uint32_t mode, session;
        state_snapshot(&mode, NULL, &session);
        if (mode == MODE_DISPLAY) {
            if (count || old_count || session != old_session)
                send_touch(points, count, session, sequence++);
            old_count = count;
            old_session = session;
            if (count >= 3) {
                if (!three_since) three_since = esp_timer_get_time();
                if (esp_timer_get_time() - three_since >= 1000000) {
                    send_touch(points, 0, session, sequence++);
                    uint32_t generation = p4desk_usb_connection_generation();
                    if (p4desk_set_mode(MODE_PAD, 0))
                        p4desk_exit_notice_begin(&exit_notice, p4desk_epoch(), generation);
                    old_count = 0;
                    three_since = 0;
                }
            } else three_since = 0;
        } else {
            old_count = 0;
            three_since = 0;
            portENTER_CRITICAL(&s_state_lock);
            if (!count) {
                s_pad_touch_active = false;
                s_pad_touch_blocked = false;
            } else if (!s_pad_touch_blocked) {
                if (!s_pad_touch_active) s_pad_touch_id = points[0].track_id;
                bool found = false;
                for (uint8_t n = 0; n < count; n++) {
                    if (points[n].track_id == s_pad_touch_id) {
                        s_pad_touch_x = points[n].x;
                        s_pad_touch_y = points[n].y;
                        found = true;
                        break;
                    }
                }
                s_pad_touch_active = found;
                if (!found) s_pad_touch_blocked = true;
            }
            portEXIT_CRITICAL(&s_state_lock);
        }
        // Keep a rejected exit request across Pad iterations. A new display
        // epoch or USB connection cancels it before it can affect that session.
        uint32_t current_mode, current_epoch;
        state_snapshot(&current_mode, &current_epoch, NULL);
        p4desk_exit_notice_service(&exit_notice, p4desk_usb_connected(), current_mode == MODE_PAD,
                                   current_epoch, p4desk_usb_connection_generation(), send_exit_notice, NULL);
        vTaskDelay(pdMS_TO_TICKS(20));
    }
}

static void log_diagnostics(int64_t now_us)
{
    // No user content or device identifiers: numeric health counters only.
    // Heap APIs and the FreeRTOS queue query hold their own allocator/queue
    // locks; all counters shared with display/USB tasks are 32-bit atomics.
    uint32_t mode;
    state_snapshot(&mode, NULL, NULL);
    const uint32_t internal = MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT;
    const uint32_t psram = MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT;
    ESP_LOGI("p4desk_diag",
        "uptime_s=%" PRId64 " mode=%s internal_free=%zu internal_largest=%zu "
        "psram_free=%zu psram_largest=%zu presented=%" PRIu32 " bad_jpeg=%" PRIu32 " "
        "parser_errors=%" PRIu32 " control_queue=%u",
        now_us / 1000000, mode == MODE_DISPLAY ? "display" : "pad",
        heap_caps_get_free_size(internal), heap_caps_get_largest_free_block(internal),
        heap_caps_get_free_size(psram), heap_caps_get_largest_free_block(psram),
        atomic_load_explicit(&s_presented_frames, memory_order_relaxed),
        atomic_load_explicit(&s_bad_jpeg, memory_order_relaxed),
        p4desk_usb_parser_errors(), (unsigned)uxQueueMessagesWaiting(s_control_queue));
}

static void watchdog_task(void *argument)
{
    (void)argument;
    int64_t last_diagnostics_us = esp_timer_get_time();
    for (;;) {
        int64_t now = esp_timer_get_time();
        portENTER_CRITICAL(&s_state_lock);
        bool expired = s_connected && s_heartbeat_us > 0 && now - s_heartbeat_us >= HEARTBEAT_TIMEOUT_US;
        if (expired) { s_heartbeat_us = 0; s_invalidated = true; }
        portEXIT_CRITICAL(&s_state_lock);
        if (expired) {
            p4desk_set_mode(MODE_PAD, 0);
            discard_packets(s_jpeg_queue);
            p4desk_usb_release_media();
            const char message[] = "{\"op\":\"request_mode\",\"mode\":\"pad\"}";
            p4desk_send_control(message, sizeof(message) - 1, 0);
        }
        if (now - last_diagnostics_us >= DIAGNOSTICS_INTERVAL_US) {
            log_diagnostics(now);
            last_diagnostics_us = now;
        }
        vTaskDelay(pdMS_TO_TICKS(100));
    }
}

void p4desk_runtime_init(board_p4_t *board)
{
    s_board = board;
    s_pad_lock = xSemaphoreCreateMutex();
    s_brightness_lock = xSemaphoreCreateMutex();
    s_refresh_event = xSemaphoreCreateBinary();
    s_control_queue = xQueueCreate(CONTROL_QUEUE_COUNT, sizeof(owned_packet_t));
    s_jpeg_queue = xQueueCreate(JPEG_QUEUE_COUNT, sizeof(owned_packet_t));
    s_pad_pixels = heap_caps_calloc(1, P4DESK_FB_BYTES, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (!s_pad_lock || !s_brightness_lock || !s_refresh_event || !s_control_queue || !s_jpeg_queue || !s_pad_pixels) abort();
    const jpeg_decode_engine_cfg_t decoder = {.intr_priority = 0, .timeout_ms = 1000};
    ESP_ERROR_CHECK(jpeg_new_decoder_engine(&decoder, &s_decoder));
    const jpeg_decode_memory_alloc_cfg_t memory = {.buffer_direction = JPEG_DEC_ALLOC_OUTPUT_BUFFER};
    s_decoded = jpeg_alloc_decoder_mem(P4DESK_WIDTH * 608 * 2, &memory, &s_decoded_capacity);
    if (!s_decoded) abort();
    const esp_lcd_dpi_panel_event_callbacks_t callbacks = {.on_refresh_done = refresh_done};
    ESP_ERROR_CHECK(esp_lcd_dpi_panel_register_event_callbacks(board->panel, &callbacks, NULL));
    atomic_store(&s_pad_dirty, true);
    if (xTaskCreatePinnedToCore(display_task, "display_owner", 8192, NULL, 5, NULL, 1) != pdPASS ||
        xTaskCreatePinnedToCore(touch_task, "touch_poll", 4096, NULL, 4, NULL, 0) != pdPASS ||
        xTaskCreate(watchdog_task, "host_deadline", 4096, NULL, 3, NULL) != pdPASS) abort();
}
