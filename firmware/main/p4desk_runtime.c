#include "p4desk_runtime.h"
#include "p4desk_hal.h"
#include "touch_exit_notice.h"
#include "pad_touch_queue.h"
#include "display_pixels.h"
#include "display_ppa.h"
#include "display_pipeline.h"
#include "display_transition.h"
#include "p4desk_lcd_frame_observer.h"

#include <inttypes.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include "driver/jpeg_decode.h"
#include "esp_attr.h"
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
#define LCD_EVENT_QUEUE_COUNT 128

typedef struct {
    p4p_header_t header;
    uint8_t *payload;
    uint32_t epoch;
    uint32_t session;
    uint32_t jpeg_rotation_degrees;
} owned_packet_t;

typedef struct {
    int64_t decode_us;
    int64_t copy_us;
    int64_t present_us;
} frame_timings_t;

typedef struct {
    p4desk_lcd_frame_event_t event;
    int64_t at_us;
} lcd_boundary_t;

typedef struct {
    bool valid, started, acknowledged;
    bool replayed, transition_unmasked;
    uint32_t mode, epoch, session, jpeg_bytes;
    uint16_t sequence;
    int64_t submitted_us;
    frame_timings_t timings;
} display_job_t;

static const char *TAG = "display_owner";
static board_p4_t *s_board;
static QueueHandle_t s_control_queue, s_jpeg_queue, s_lcd_events;
static SemaphoreHandle_t s_pad_lock, s_brightness_lock;
static uint16_t *s_pad_pixels;
static _Atomic(TaskHandle_t) s_pad_writer;
static _Atomic(TaskHandle_t) s_display_owner_task;
static jpeg_decoder_handle_t s_decoder;
static uint8_t *s_decoded;
static size_t s_decoded_capacity;
static size_t s_lcd_capacity[P4DESK_FB_COUNT];
static p4desk_ppa_t *s_ppa;
static atomic_bool s_pad_dirty;
static atomic_bool s_lcd_ownership_lost;
static atomic_uint s_presented_frames, s_bad_jpeg;
static atomic_uint s_jpeg_received, s_jpeg_dropped, s_frame_ack_dropped;
static portMUX_TYPE s_state_lock = portMUX_INITIALIZER_UNLOCKED;
static uint32_t s_mode = MODE_PAD, s_epoch = 1, s_session, s_jpeg_rotation_degrees;
static bool s_connected, s_invalidated = true, s_time_valid;
static int64_t s_heartbeat_us;
static p4desk_display_transition_t s_display_transition;
static atomic_int s_battery_voltage_mv = -1;
static atomic_uchar s_brightness = 75;
static atomic_bool s_backlight_on = true;
static bool s_pad_touch_active, s_pad_touch_blocked;
static uint8_t s_pad_touch_id;
static int32_t s_pad_touch_x, s_pad_touch_y;
static p4desk_pad_touch_queue_t s_pad_touch_queue;
static p4desk_touch_frame_t s_raw_touch;
static bool s_raw_touch_valid;

static void display_wake(void)
{
    TaskHandle_t task = atomic_load(&s_display_owner_task);
    if (task) xTaskNotifyGive(task);
}

static void state_image_snapshot(uint32_t *mode, uint32_t *epoch, uint32_t *session,
                                 uint32_t *jpeg_rotation_degrees)
{
    portENTER_CRITICAL(&s_state_lock);
    if (mode) *mode = s_mode;
    if (epoch) *epoch = s_epoch;
    if (session) *session = s_session;
    if (jpeg_rotation_degrees) *jpeg_rotation_degrees = s_jpeg_rotation_degrees;
    portEXIT_CRITICAL(&s_state_lock);
}

static void state_snapshot(uint32_t *mode, uint32_t *epoch, uint32_t *session)
{
    state_image_snapshot(mode, epoch, session, NULL);
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

bool p4desk_arm_display_transition(uint32_t duration_ms)
{
    portENTER_CRITICAL(&s_state_lock);
    bool armed = s_mode == MODE_PAD && s_connected &&
                 p4dt_arm(&s_display_transition, duration_ms);
    portEXIT_CRITICAL(&s_state_lock);
    return armed;
}

void p4desk_cancel_display_transition(void)
{
    portENTER_CRITICAL(&s_state_lock);
    p4dt_cancel(&s_display_transition);
    portEXIT_CRITICAL(&s_state_lock);
    display_wake();
}

bool p4desk_display_transition_pending(void)
{
    portENTER_CRITICAL(&s_state_lock);
    bool pending = p4dt_pending(&s_display_transition);
    portEXIT_CRITICAL(&s_state_lock);
    return pending;
}

static bool display_transition_active(uint32_t epoch)
{
    portENTER_CRITICAL(&s_state_lock);
    bool active = p4dt_active(&s_display_transition, epoch);
    portEXIT_CRITICAL(&s_state_lock);
    return active;
}

bool p4desk_set_mode(uint32_t mode, uint32_t session)
{
    return p4desk_set_mode_with_jpeg_rotation(mode, session, 0);
}

uint32_t p4desk_direct_jpeg_rotation_degrees(void)
{
    return P4DESK_DISPLAY_ROTATION_DEGREES;
}

bool p4desk_set_mode_with_jpeg_rotation(uint32_t mode, uint32_t session, uint32_t jpeg_rotation_degrees)
{
    if (mode > MODE_DISPLAY) return false;
    if ((mode == MODE_PAD && jpeg_rotation_degrees != 0) ||
        (jpeg_rotation_degrees != 0 && jpeg_rotation_degrees != p4desk_direct_jpeg_rotation_degrees())) return false;
    bool changed = false;
    portENTER_CRITICAL(&s_state_lock);
    if (mode == MODE_DISPLAY && !s_connected) {
        portEXIT_CRITICAL(&s_state_lock);
        return false;
    }
    if (s_mode != mode || (mode == MODE_DISPLAY &&
        (s_session != session || s_jpeg_rotation_degrees != jpeg_rotation_degrees))) {
        s_mode = mode;
        s_session = session;
        s_jpeg_rotation_degrees = jpeg_rotation_degrees;
        ++s_epoch;
        if (mode == MODE_DISPLAY) p4dt_bind(&s_display_transition, s_epoch);
        s_invalidated = true;
        s_pad_touch_active = false;
        s_pad_touch_blocked = true; // Re-arm after all contacts are released.
        p4desk_pad_touch_queue_reset(&s_pad_touch_queue);
        changed = true;
    }
    if (mode == MODE_PAD) p4dt_cancel(&s_display_transition);
    if (mode == MODE_DISPLAY) s_heartbeat_us = esp_timer_get_time();
    portEXIT_CRITICAL(&s_state_lock);
    if (changed) {
        discard_packets(s_jpeg_queue);
        // The USB owner must consume an in-flight message to its boundary.
        // Its original epoch discards stale JPEGs after this mode change.
        atomic_store(&s_pad_dirty, true);
        display_wake();
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
    uint32_t mode, current_epoch, session, jpeg_rotation_degrees;
    state_image_snapshot(&mode, &current_epoch, &session, &jpeg_rotation_degrees);
    if (header->kind == P4P_KIND_JPEG && (mode != MODE_DISPLAY || epoch != current_epoch)) return;
    if (header->kind == P4P_KIND_JPEG)
        atomic_fetch_add_explicit(&s_jpeg_received, 1, memory_order_relaxed);
    owned_packet_t packet = {.header = *header, .epoch = epoch, .session = session,
                             .jpeg_rotation_degrees = jpeg_rotation_degrees};
    packet.payload = heap_caps_malloc(header->payload_length, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (!packet.payload) {
        if (header->kind == P4P_KIND_JPEG)
            atomic_fetch_add_explicit(&s_jpeg_dropped, 1, memory_order_relaxed);
        packet_rejected(header, "out_of_memory");
        return;
    }
    memcpy(packet.payload, payload, header->payload_length);
    QueueHandle_t queue = header->kind == P4P_KIND_JPEG ? s_jpeg_queue : s_control_queue;
    if (xQueueSend(queue, &packet, 0) == pdTRUE) {
        if (header->kind == P4P_KIND_JPEG) display_wake();
        return;
    }
    if (header->kind == P4P_KIND_JPEG) {
        owned_packet_t stale;
        if (xQueueReceive(queue, &stale, 0) == pdTRUE) {
            free(stale.payload);
            atomic_fetch_add_explicit(&s_jpeg_dropped, 1, memory_order_relaxed);
        }
        if (xQueueSend(queue, &packet, 0) == pdTRUE) { display_wake(); return; }
        atomic_fetch_add_explicit(&s_jpeg_dropped, 1, memory_order_relaxed);
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
    display_wake();
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
        display_wake();
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

bool p4desk_poll_pad_touch(p4desk_pad_touch_event_t *event)
{
    if (!event) return false;
    portENTER_CRITICAL(&s_state_lock);
    bool ready = s_mode == MODE_PAD && p4desk_pad_touch_queue_pop(&s_pad_touch_queue, event);
    portEXIT_CRITICAL(&s_state_lock);
    return ready;
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

static bool IRAM_ATTR dma_boundary(esp_lcd_panel_handle_t panel,
                                  const p4desk_lcd_frame_event_t *event, void *context)
{
    (void)panel; (void)context;
    const lcd_boundary_t boundary = {.event = *event, .at_us = esp_timer_get_time()};
    BaseType_t wake = pdFALSE;
    // Preserve every boundary, including repeated scans. An overflow makes
    // ownership uncertain; the display task must stop before reusing a buffer.
    if (xQueueSendFromISR(s_lcd_events, &boundary, &wake) != pdTRUE)
        atomic_store_explicit(&s_lcd_ownership_lost, true, memory_order_relaxed);
    TaskHandle_t task = atomic_load(&s_display_owner_task);
    if (task) vTaskNotifyGiveFromISR(task, &wake);
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

static bool decode_frame(const owned_packet_t *packet, uint16_t *target, size_t target_capacity,
                          frame_timings_t *timings)
{
    // Include header validation and synchronous hardware decoding in this
    // phase. No image bytes or user content enter the diagnostic metadata.
    const int64_t decode_started_us = esp_timer_get_time();
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
    const bool host_rotated = packet->jpeg_rotation_degrees == P4DESK_DISPLAY_ROTATION_DEGREES;
    const bool direct = !gray && host_rotated && stride == P4DESK_WIDTH;
    uint8_t *output = direct ? (uint8_t *)target : s_decoded;
    size_t capacity = direct ? target_capacity : s_decoded_capacity;
    if (required > capacity || capacity > UINT32_MAX) return reject_jpeg();
    jpeg_decode_cfg_t config = {
        .output_format = gray ? JPEG_DECODE_OUT_FORMAT_GRAY : JPEG_DECODE_OUT_FORMAT_RGB565,
        // BGR is the driver's little-endian output setting, preserving native
        // RGB565 words for the RGB element-order DSI panel. The project-local
        // jpeg_full_range component replaces BT.601's studio-range matrix for
        // JPEG RGB output with JFIF full-range coefficients. It does not swap
        // red/blue, change Pad pixels, or add a framebuffer copy.
        .rgb_order = JPEG_DEC_RGB_ELEMENT_ORDER_BGR, .conv_std = JPEG_YUV_RGB_CONV_STD_BT601,
    };
    uint32_t written = 0;
    esp_err_t decoded = jpeg_decoder_process(s_decoder, &config, packet->payload, packet->header.payload_length,
                                             output, capacity, &written);
    if (decoded == ESP_ERR_TIMEOUT && direct) {
        ESP_LOGE(TAG, "Direct JPEG completion deadline missed; buffers remain owned");
        abort();
    }
    if (decoded != ESP_OK || written != required) return reject_jpeg();
    timings->decode_us = esp_timer_get_time() - decode_started_us;
    if (!frame_current(packet)) return false;
    if (direct) {
        // The host rotated the visible 1024x600 image before JPEG encoding.
        // Padding is written only to the explicitly allocated 608-row tail;
        // LCD DMA still reads exactly 600 rows. No pixel copy is needed.
        timings->copy_us = 0;
        return true;
    }
    const bool rotate = P4DESK_DISPLAY_ROTATION_DEGREES == 180 && !host_rotated;
    // Rotate only the visible 600 rows while copying to a BUILDING buffer.
    // MCU padding (e.g. 608 decoded rows) never enters the visible image.
    const int64_t copy_started_us = esp_timer_get_time();
    if (gray)
        p4desk_pixels_copy_gray565(target, s_decoded, P4DESK_WIDTH, P4DESK_HEIGHT,
                                  stride, rotate);
    else {
        static bool ppa_logged, fallback_logged;
        esp_err_t copied = s_ppa ? p4desk_ppa_copy_rgb565(s_ppa, target, P4DESK_FB_BYTES,
            (const uint16_t *)s_decoded, s_decoded_capacity, stride, rows,
            rotate, 1000) : ESP_ERR_NOT_SUPPORTED;
        if (copied == ESP_ERR_TIMEOUT || copied == ESP_ERR_INVALID_STATE) {
            // As with a missed LCD refresh, do not reuse DMA-owned buffers.
            ESP_LOGE(TAG, "PPA completion deadline missed; buffers remain owned");
            abort();
        }
        if (copied == ESP_OK) {
            if (!ppa_logged) {
                ESP_LOGI(TAG, "RGB565 crop/rotation uses PPA SRM, visible rows=%d", P4DESK_HEIGHT);
                ppa_logged = true;
            }
        } else {
            if (!fallback_logged) {
                ESP_LOGW(TAG, "PPA rejected copy (%s); CPU fallback active", esp_err_to_name(copied));
                fallback_logged = true;
            }
            p4desk_pixels_copy_rgb565(target, (const uint16_t *)s_decoded, P4DESK_WIDTH,
                                     P4DESK_HEIGHT, stride, rotate);
        }
    }
    timings->copy_us = esp_timer_get_time() - copy_started_us;
    return frame_current(packet);
}

static bool job_current(const display_job_t *job)
{
    uint32_t mode, epoch, session;
    state_snapshot(&mode, &epoch, &session);
    return job->valid && job->mode == mode && job->epoch == epoch &&
           (mode == MODE_PAD || job->session == session);
}

static void acknowledge_job(display_job_t *job, int64_t completed_us)
{
    job->acknowledged = true;
    atomic_fetch_add_explicit(&s_presented_frames, 1, memory_order_relaxed);
    if (!job_current(job)) return;
    if (job->mode == MODE_PAD) { apply_backlight(); return; }
    portENTER_CRITICAL(&s_state_lock);
    p4dt_presented(&s_display_transition, job->epoch, job->transition_unmasked);
    portEXIT_CRITICAL(&s_state_lock);
    // Re-decoding a retained JPEG advances animation on static captures. Its
    // original wire sequence is acknowledged only for the real received frame.
    if (job->replayed) return;
    job->timings.present_us = completed_us - job->submitted_us;
    char message[384];
    int length = snprintf(message, sizeof(message),
        "{\"op\":\"frame_presented\",\"session\":%" PRIu32 ",\"sequence\":%u,\"device_us\":%" PRId64
        ",\"jpeg_bytes\":%" PRIu32 ",\"decode_us\":%" PRId64 ",\"copy_us\":%" PRId64 ",\"present_us\":%" PRId64 "}",
        job->session, job->sequence, completed_us, job->jpeg_bytes,
        job->timings.decode_us, job->timings.copy_us, job->timings.present_us);
    if (length <= 0 || length >= sizeof(message) ||
        !p4desk_send_control(message, length, job->sequence))
        atomic_fetch_add_explicit(&s_frame_ack_dropped, 1, memory_order_relaxed);
}

typedef struct {
    p4dp_owner_t owner;
    display_job_t jobs[P4DESK_FB_COUNT];
    uint32_t last_counter;
    int64_t last_boundary_us;
    bool counter_valid;
    owned_packet_t retained_jpeg; // One encoded frame, at most MAX_JPEG, never an extra pixel buffer.
} display_pipeline_t;

static void consume_boundary(display_pipeline_t *pipeline, const lcd_boundary_t *boundary)
{
    const p4desk_lcd_frame_event_t *event = &boundary->event;
    if (atomic_load_explicit(&s_lcd_ownership_lost, memory_order_relaxed) ||
        event->completed_index >= P4DESK_FB_COUNT || event->next_index >= P4DESK_FB_COUNT ||
        event->completed_fb != s_board->framebuffers[event->completed_index] ||
        event->next_fb != s_board->framebuffers[event->next_index] ||
        (pipeline->counter_valid && (uint32_t)(event->counter - pipeline->last_counter) != 1) ||
        !p4dp_dma_complete(&pipeline->owner, event->completed_index, event->next_index)) {
        ESP_LOGE(TAG, "LCD DMA boundary sequence lost; buffers remain owned");
        abort();
    }
    pipeline->last_counter = event->counter;
    pipeline->counter_valid = true;
    pipeline->last_boundary_us = boundary->at_us;
    display_job_t *completed = &pipeline->jobs[event->completed_index];
    if (completed->valid && completed->started && !completed->acknowledged)
        acknowledge_job(completed, boundary->at_us);
    if (event->completed_index != event->next_index)
        memset(completed, 0, sizeof(*completed));
    display_job_t *next = &pipeline->jobs[event->next_index];
    if (next->valid) next->started = true;
}

static void drain_boundaries(display_pipeline_t *pipeline)
{
    lcd_boundary_t boundary;
    while (xQueueReceive(s_lcd_events, &boundary, 0) == pdTRUE) consume_boundary(pipeline, &boundary);
    if (atomic_load_explicit(&s_lcd_ownership_lost, memory_order_relaxed) ||
        esp_timer_get_time() - pipeline->last_boundary_us > 1000000) {
        ESP_LOGE(TAG, "LCD DMA completion deadline missed; buffers remain owned");
        abort();
    }
}

static void submit_ready(display_pipeline_t *pipeline)
{
    int index = pipeline->owner.ready;
    if (index < 0) return;
    display_job_t *job = &pipeline->jobs[index];
    if (!job_current(job)) {
        if (job->mode == MODE_PAD) atomic_store(&s_pad_dirty, true);
        if (!p4dp_discard_prepared(&pipeline->owner, index)) abort();
        memset(job, 0, sizeof(*job));
        return;
    }
    if (pipeline->owner.pending >= 0) return;
    if (!p4dp_submit(&pipeline->owner, index)) abort();
    job->submitted_us = esp_timer_get_time();
    // The driver synchronizes cache and publishes only this PENDING buffer.
    // Actual completed/next identities, rather than a VSYNC count, release it.
    ESP_ERROR_CHECK(esp_lcd_panel_draw_bitmap(s_board->panel, 0, 0, P4DESK_WIDTH,
                                             P4DESK_HEIGHT, s_board->framebuffers[index]));
}

static bool take_latest_jpeg(owned_packet_t *packet)
{
    if (xQueueReceive(s_jpeg_queue, packet, 0) != pdTRUE) return false;
    owned_packet_t newer;
    // Bounded snapshot avoids starving the display task if the host is busy.
    for (int n = 1; n < JPEG_QUEUE_COUNT; n++) {
        if (xQueueReceive(s_jpeg_queue, &newer, 0) != pdTRUE) break;
        free(packet->payload); *packet = newer;
        atomic_fetch_add_explicit(&s_jpeg_dropped, 1, memory_order_relaxed);
    }
    return true;
}

static bool prepare_buffer(display_pipeline_t *pipeline)
{
    if (pipeline->retained_jpeg.payload &&
        (!frame_current(&pipeline->retained_jpeg) ||
         !display_transition_active(pipeline->retained_jpeg.epoch))) {
        free(pipeline->retained_jpeg.payload);
        memset(&pipeline->retained_jpeg, 0, sizeof(pipeline->retained_jpeg));
    }
    int index = p4dp_reserve(&pipeline->owner);
    if (index < 0) return false;
    display_job_t *job = &pipeline->jobs[index];
    memset(job, 0, sizeof(*job));
    state_snapshot(&job->mode, &job->epoch, &job->session);
    bool prepared = false;
    if (job->mode == MODE_PAD) {
        if (atomic_exchange(&s_pad_dirty, false)) {
            xSemaphoreTake(s_pad_lock, portMAX_DELAY);
            // Same owner/client as JPEG: hold immutable Pad source until SRM
            // completes, and only write this reserved BUILDING LCD buffer.
            const int64_t copy_started_us = esp_timer_get_time();
            esp_err_t copied = s_ppa ? p4desk_ppa_copy_rgb565(s_ppa,
                s_board->framebuffers[index], P4DESK_FB_BYTES, s_pad_pixels,
                P4DESK_FB_BYTES, P4DESK_WIDTH, P4DESK_HEIGHT,
                P4DESK_DISPLAY_ROTATION_DEGREES == 180, 1000) : ESP_ERR_NOT_SUPPORTED;
            if (copied == ESP_ERR_TIMEOUT || copied == ESP_ERR_INVALID_STATE) {
                ESP_LOGE(TAG, "Pad PPA completion deadline missed; buffers remain owned");
                abort();
            }
            if (copied != ESP_OK) {
                static bool fallback_logged;
                if (!fallback_logged) {
                    ESP_LOGW(TAG, "Pad PPA rejected copy (%s); CPU fallback active", esp_err_to_name(copied));
                    fallback_logged = true;
                }
                p4desk_pixels_copy_rgb565(s_board->framebuffers[index], s_pad_pixels,
                                         P4DESK_WIDTH, P4DESK_HEIGHT, P4DESK_WIDTH,
                                         P4DESK_DISPLAY_ROTATION_DEGREES == 180);
            }
            job->timings.copy_us = esp_timer_get_time() - copy_started_us;
            xSemaphoreGive(s_pad_lock);
            prepared = true;
        }
    } else {
        owned_packet_t packet = {0};
        const bool received = take_latest_jpeg(&packet);
        if (!received && pipeline->retained_jpeg.payload) {
            packet = pipeline->retained_jpeg;
            job->replayed = true;
        }
        if (packet.payload) {
            job->epoch = packet.epoch; job->session = packet.session;
            job->sequence = packet.header.sequence; job->jpeg_bytes = packet.header.payload_length;
            prepared = frame_current(&packet) &&
                decode_frame(&packet, s_board->framebuffers[index], s_lcd_capacity[index], &job->timings);
            if (prepared) {
                uint32_t elapsed_ms = 0, duration_ms = 0;
                portENTER_CRITICAL(&s_state_lock);
                bool reveal = p4dt_sample(&s_display_transition, packet.epoch,
                    esp_timer_get_time(), &elapsed_ms, &duration_ms);
                portEXIT_CRITICAL(&s_state_lock);
                if (reveal) {
                    const int64_t paint_started_us = esp_timer_get_time();
                    // Rust reuses the Pad glass compositor. This buffer is
                    // exclusively BUILDING and both hardware decoders are done.
                    prepared = rust_p4desk_display_reveal(s_board->framebuffers[index],
                        P4DESK_WIDTH * P4DESK_HEIGHT, elapsed_ms, duration_ms);
                    job->timings.copy_us += esp_timer_get_time() - paint_started_us;
                    job->transition_unmasked = elapsed_ms >= duration_ms;
                    if (prepared && received) {
                        free(pipeline->retained_jpeg.payload);
                        pipeline->retained_jpeg = packet;
                        packet.payload = NULL; // Transfer, not a JPEG copy.
                    }
                }
            }
            if (received) free(packet.payload);
        }
    }
    job->valid = prepared;
    // JPEG/PPA writes have completed before BUILDING can be cancelled.
    if (!prepared || !job_current(job)) {
        if (prepared && job->mode == MODE_PAD) atomic_store(&s_pad_dirty, true);
        if (!p4dp_discard_prepared(&pipeline->owner, index)) abort();
        memset(job, 0, sizeof(*job));
        return false;
    }
    if (!p4dp_ready(&pipeline->owner, index)) abort();
    return true;
}

static void display_task(void *argument)
{
    (void)argument;
    _Static_assert(P4DESK_FB_COUNT == P4DP_BUFFER_COUNT, "LCD ownership helper must match board buffers");
    display_pipeline_t pipeline = {.last_boundary_us = esp_timer_get_time()};
    p4dp_init(&pipeline.owner, 0);
    atomic_store(&s_display_owner_task, xTaskGetCurrentTaskHandle());
    for (;;) {
        drain_boundaries(&pipeline);
        submit_ready(&pipeline);
        bool prepared = prepare_buffer(&pipeline);
        // Drain first so a previous pending selection has actually started.
        // Preparation may overlap DMA, but only FREE buffers can be written.
        drain_boundaries(&pipeline);
        submit_ready(&pipeline);
        // USB and DMA both wake this owner; a JPEG arriving just after the
        // queue check must not wait for another LCD frame boundary.
        if (!prepared) ulTaskNotifyTake(pdTRUE, pdMS_TO_TICKS(20));
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
    bool display_touch_blocked = true;
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
            // Apply the calibrated GT911 correction before all input consumers.
            // Clamp first; the generic touch mirror uses width-x, not width-1-x.
#if P4DESK_TOUCH_ROTATION_DEGREES == 180
            points[n].x = (P4DESK_WIDTH - 1) - points[n].x;
            points[n].y = (P4DESK_HEIGHT - 1) - points[n].y;
#endif
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
        uint32_t mode, epoch, session;
        state_snapshot(&mode, &epoch, &session);
        if (mode == MODE_DISPLAY) {
            bool transitioning = display_transition_active(epoch);
            if (transitioning || session != old_session) display_touch_blocked = true;
            if (!transitioning && !count) display_touch_blocked = false;
            uint8_t reported_count = display_touch_blocked ? 0 : count;
            if (reported_count || old_count || session != old_session)
                send_touch(points, reported_count, session, sequence++);
            old_count = reported_count;
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
            display_touch_blocked = true;
            three_since = 0;
            portENTER_CRITICAL(&s_state_lock);
            // Mode can change after state_snapshot; never enqueue stale Pad input.
            if (s_mode == MODE_PAD) {
                bool was_active = s_pad_touch_active;
                int32_t old_x = s_pad_touch_x, old_y = s_pad_touch_y;
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
                uint32_t kind = 0;
                if (!was_active && s_pad_touch_active) kind = P4DESK_PAD_TOUCH_DOWN;
                else if (was_active && !s_pad_touch_active)
                    kind = count ? P4DESK_PAD_TOUCH_CANCEL : P4DESK_PAD_TOUCH_UP;
                else if (was_active && (old_x != s_pad_touch_x || old_y != s_pad_touch_y))
                    kind = P4DESK_PAD_TOUCH_MOVE;
                if (kind && !p4desk_pad_touch_queue_push(&s_pad_touch_queue,
                    (p4desk_pad_touch_event_t){.kind = kind, .x = s_pad_touch_x, .y = s_pad_touch_y})) {
                    s_pad_touch_active = false;
                    s_pad_touch_blocked = true;
                }
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
        "parser_errors=%" PRIu32 " control_queue=%u jpeg_received=%" PRIu32 " "
        "jpeg_dropped=%" PRIu32 " frame_ack_dropped=%" PRIu32,
        now_us / 1000000, mode == MODE_DISPLAY ? "display" : "pad",
        heap_caps_get_free_size(internal), heap_caps_get_largest_free_block(internal),
        heap_caps_get_free_size(psram), heap_caps_get_largest_free_block(psram),
        atomic_load_explicit(&s_presented_frames, memory_order_relaxed),
        atomic_load_explicit(&s_bad_jpeg, memory_order_relaxed),
        p4desk_usb_parser_errors(), (unsigned)uxQueueMessagesWaiting(s_control_queue),
        atomic_load_explicit(&s_jpeg_received, memory_order_relaxed),
        atomic_load_explicit(&s_jpeg_dropped, memory_order_relaxed),
        atomic_load_explicit(&s_frame_ack_dropped, memory_order_relaxed));
}

int32_t p4desk_battery_voltage_mv(void)
{
    return atomic_load_explicit(&s_battery_voltage_mv, memory_order_relaxed);
}

static void watchdog_task(void *argument)
{
    (void)argument;
    int64_t last_diagnostics_us = esp_timer_get_time();
    int64_t last_battery_us = last_diagnostics_us - 2000000;
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
        if (now - last_battery_us >= 2000000) {
            // ADC work stays off the UI and display-owner paths. Invalid samples
            // clear stale telemetry; no USB state is used to infer charging.
            atomic_store_explicit(&s_battery_voltage_mv, board_p4_battery_voltage_mv(), memory_order_relaxed);
            last_battery_us = now;
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
    for (int index = 0; index < P4DESK_FB_COUNT; index++) {
        ESP_ERROR_CHECK(p4desk_lcd_frame_buffer_capacity(board->panel, board->framebuffers[index],
                                                        &s_lcd_capacity[index]));
        if (s_lcd_capacity[index] < P4DESK_WIDTH * 608 * sizeof(uint16_t)) abort();
    }
    s_pad_lock = xSemaphoreCreateMutex();
    s_brightness_lock = xSemaphoreCreateMutex();
    s_lcd_events = xQueueCreate(LCD_EVENT_QUEUE_COUNT, sizeof(lcd_boundary_t));
    s_control_queue = xQueueCreate(CONTROL_QUEUE_COUNT, sizeof(owned_packet_t));
    s_jpeg_queue = xQueueCreate(JPEG_QUEUE_COUNT, sizeof(owned_packet_t));
    s_pad_pixels = heap_caps_calloc(1, P4DESK_FB_BYTES, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (!s_pad_lock || !s_brightness_lock || !s_lcd_events || !s_control_queue || !s_jpeg_queue || !s_pad_pixels) abort();
    const jpeg_decode_engine_cfg_t decoder = {.intr_priority = 0, .timeout_ms = 1000};
    ESP_ERROR_CHECK(jpeg_new_decoder_engine(&decoder, &s_decoder));
    const jpeg_decode_memory_alloc_cfg_t memory = {.buffer_direction = JPEG_DEC_ALLOC_OUTPUT_BUFFER};
    s_decoded = jpeg_alloc_decoder_mem(P4DESK_WIDTH * 608 * 2, &memory, &s_decoded_capacity);
    if (!s_decoded) abort();
    esp_err_t ppa_ready = p4desk_ppa_create(&s_ppa);
    if (ppa_ready != ESP_OK)
        ESP_LOGW(TAG, "PPA client unavailable (%s); CPU fallback active", esp_err_to_name(ppa_ready));
    ESP_ERROR_CHECK(p4desk_lcd_frame_observer_register(board->panel, dma_boundary, NULL));
    atomic_store(&s_pad_dirty, true);
    if (xTaskCreatePinnedToCore(display_task, "display_owner", 8192, NULL, 5, NULL, 1) != pdPASS ||
        xTaskCreatePinnedToCore(touch_task, "touch_poll", 4096, NULL, 4, NULL, 0) != pdPASS ||
        xTaskCreate(watchdog_task, "host_deadline", 4096, NULL, 3, NULL) != pdPASS) abort();
}
