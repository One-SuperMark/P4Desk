#include "p4desk_runtime.h"
#include "p4desk_hal.h"
#include "usb_rx_stream.h"
#include "usb_tx_queue.h"

#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>
#include "esp_check.h"
#include "esp_heap_caps.h"
#include "esp_log.h"
#include "esp_private/usb_phy.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/queue.h"
#include "freertos/task.h"
#include "tusb.h"

typedef struct {
    uint16_t usage;
    uint32_t generation;
} media_packet_t;

static const char *TAG = "p4desk_usb";
static usb_phy_handle_t s_phy;
static p4desk_tx_queue_t s_tx_priority, s_tx_events, s_tx_release;
static QueueHandle_t s_media;
static p4desk_rx_stream_t s_stream;
static atomic_bool s_media_clear_requested;
static atomic_uint s_parser_errors;
static atomic_uint s_connection_generation;
static p4desk_tx_packet_t s_transmitting;
static size_t s_transmitted;
static uint16_t s_media_usage;
static bool s_media_release;

static void on_message(const p4p_header_t *header, const uint8_t *payload, uint32_t epoch, void *context)
{
    (void)context;
    p4desk_receive_message(header, payload, epoch);
}

void p4desk_usb_release_media(void)
{
    // Only usb_task accesses TinyUSB or HID report state.
    atomic_store(&s_media_clear_requested, true);
}

uint32_t p4desk_usb_parser_errors(void)
{
    // The parser belongs to usb_task. Diagnostics read only this atomic copy.
    return atomic_load_explicit(&s_parser_errors, memory_order_relaxed);
}

uint32_t p4desk_usb_connection_generation(void)
{
    return atomic_load_explicit(&s_connection_generation, memory_order_relaxed);
}

static bool queue_send(void *context, const p4desk_tx_packet_t *packet)
{
    return xQueueSend((QueueHandle_t)context, packet, 0) == pdTRUE;
}

static bool queue_receive(void *context, p4desk_tx_packet_t *packet)
{
    return xQueueReceive((QueueHandle_t)context, packet, 0) == pdTRUE;
}

static p4desk_tx_queue_t make_queue(UBaseType_t depth)
{
    return (p4desk_tx_queue_t){
        .context = xQueueCreate(depth, sizeof(p4desk_tx_packet_t)),
        .send = queue_send, .receive = queue_receive, .dispose = free,
    };
}

static bool queue_json(const char *json, size_t length, uint16_t sequence,
                        p4desk_tx_queue_t *queue, bool replace_oldest)
{
    uint32_t generation = p4desk_usb_connection_generation();
    if (!json || !length || length > P4P_MAX_CONTROL || sequence > 1023 ||
        !queue->context || !p4desk_usb_connected()) return false;
    p4desk_tx_packet_t packet = {.length = P4P_HEADER_SIZE + length, .generation = generation};
    packet.bytes = heap_caps_malloc(packet.length, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (!packet.bytes) return false;
    const p4p_header_t header = {
        .crc16 = p4p_crc16((const uint8_t *)json, length),
        .kind = P4P_KIND_CONTROL, .flags = P4P_CRC_PRESENT,
        .sequence = sequence, .payload_length = length,
    };
    if (!p4p_encode_header(&header, packet.bytes)) { free(packet.bytes); return false; }
    memcpy(packet.bytes + P4P_HEADER_SIZE, json, length);
    if (!p4desk_usb_connected() || generation != p4desk_usb_connection_generation()) {
        free(packet.bytes);
        return false;
    }
    return p4desk_tx_queue_put(queue, packet, replace_oldest);
}

bool p4desk_usb_queue_json(const char *json, size_t length, uint16_t sequence, bool priority)
{
    return queue_json(json, length, sequence, priority ? &s_tx_priority : &s_tx_events, !priority);
}

bool p4desk_usb_queue_touch(const char *json, size_t length, uint16_t sequence, bool released)
{
    // Only touch_task produces release messages. A newer release replaces a
    // pending one independently of ACK congestion; active TX retains ownership.
    return queue_json(json, length, sequence, released ? &s_tx_release : &s_tx_events, true);
}

void p4desk_usb_queue_media(uint16_t usage)
{
    uint32_t generation = p4desk_usb_connection_generation();
    if (!s_media || !p4desk_usb_connected() || !usage || usage > 0x03ff) return;
    const media_packet_t packet = {.usage = usage, .generation = generation};
    xQueueSend(s_media, &packet, 0);
}

static void media_clear(void)
{
    media_packet_t packet;
    while (xQueueReceive(s_media, &packet, 0) == pdTRUE) {}
    // A press may already have reached the host. Preserve a zero report until
    // the endpoint is ready, including across suspend/resume or a failed IN.
    s_media_release = true;
    s_media_usage = 0;
}

static void tx_clear(void)
{
    if (s_transmitting.bytes) free(s_transmitting.bytes);
    memset(&s_transmitting, 0, sizeof(s_transmitting));
    s_transmitted = 0;
    p4desk_tx_queue_clear(&s_tx_release);
    p4desk_tx_queue_clear(&s_tx_priority);
    p4desk_tx_queue_clear(&s_tx_events);
    media_clear();
}

static void tx_service(void)
{
    if (!tud_mounted()) return;
    if (!s_transmitting.bytes) {
        if (!p4desk_tx_select_next(&s_tx_release, &s_tx_priority, &s_tx_events,
                                   p4desk_usb_connection_generation(), &s_transmitting)) return;
        s_transmitted = 0;
    }
    uint32_t available = tud_vendor_n_write_available(0);
    if (!available) return;
    size_t remaining = s_transmitting.length - s_transmitted;
    if (remaining > available) remaining = available;
    uint32_t written = tud_vendor_n_write(0, s_transmitting.bytes + s_transmitted, remaining);
    s_transmitted += written;
    tud_vendor_n_write_flush(0);
    if (s_transmitted == s_transmitting.length) {
        free(s_transmitting.bytes);
        memset(&s_transmitting, 0, sizeof(s_transmitting));
        s_transmitted = 0;
    }
}

static void media_service(void)
{
    if (!tud_mounted() || !tud_hid_ready()) return;
    if (s_media_release) {
        uint16_t release = 0;
        if (tud_hid_report(1, &release, sizeof(release))) {
            s_media_release = false;
            s_media_usage = 0;
        }
        return;
    }
    media_packet_t packet;
    for (;;) {
        if (xQueuePeek(s_media, &packet, 0) != pdTRUE) return;
        if (packet.generation == p4desk_usb_connection_generation()) break;
        // Drop actions that raced disconnect cleanup; keep zero-release first.
        xQueueReceive(s_media, &packet, 0);
    }
    if (tud_hid_report(1, &packet.usage, sizeof(packet.usage))) {
        xQueueReceive(s_media, &packet, 0);
        s_media_usage = packet.usage;
        s_media_release = true;
    }
}

void tud_vendor_rx_cb(uint8_t instance, const uint8_t *buffer, uint16_t size)
{
    (void)buffer; (void)size;
    static uint8_t bytes[4096] __attribute__((aligned(64)));
    while (tud_vendor_n_available(instance)) {
        uint32_t read = tud_vendor_n_read(instance, bytes, sizeof(bytes));
        if (!read) break;
        p4desk_rx_stream_feed(&s_stream, bytes, read, esp_timer_get_time(), p4desk_epoch());
    }
}

void tud_mount_cb(void)
{
    atomic_fetch_add_explicit(&s_connection_generation, 1, memory_order_relaxed);
    p4desk_rx_stream_reset(&s_stream);
    tx_clear();
    p4desk_usb_mount_changed(true);
    ESP_LOGI(TAG, "vendor and media connected (%s)", tud_speed_get() == TUSB_SPEED_HIGH ? "HS" : "FS");
}

void tud_umount_cb(void)
{
    atomic_fetch_add_explicit(&s_connection_generation, 1, memory_order_relaxed);
    p4desk_usb_mount_changed(false);
    p4desk_rx_stream_reset(&s_stream);
    tx_clear();
    ESP_LOGI(TAG, "USB disconnected; Pad active");
}

void tud_suspend_cb(bool remote_wakeup_enabled)
{
    (void)remote_wakeup_enabled;
    atomic_fetch_add_explicit(&s_connection_generation, 1, memory_order_relaxed);
    p4desk_usb_mount_changed(false);
    p4desk_rx_stream_reset(&s_stream);
    tx_clear();
    tud_vendor_n_read_flush(0);
    tud_vendor_n_write_clear(0);
}

void tud_resume_cb(void)
{
    atomic_fetch_add_explicit(&s_connection_generation, 1, memory_order_relaxed);
    p4desk_rx_stream_reset(&s_stream);
    tx_clear();
    p4desk_usb_mount_changed(true);
}

void tud_hid_report_failed_cb(uint8_t instance, hid_report_type_t type,
                              const uint8_t *report, uint16_t transferred)
{
    (void)instance; (void)report; (void)transferred;
    if (type == HID_REPORT_TYPE_INPUT) s_media_release = true;
}

uint16_t tud_hid_get_report_cb(uint8_t instance, uint8_t report_id, hid_report_type_t report_type,
                             uint8_t *buffer, uint16_t requested)
{
    (void)instance;
    if (report_id != 1 || report_type != HID_REPORT_TYPE_INPUT || requested < sizeof(s_media_usage)) return 0;
    memcpy(buffer, &s_media_usage, sizeof(s_media_usage));
    return sizeof(s_media_usage);
}

void tud_hid_set_report_cb(uint8_t instance, uint8_t report_id, hid_report_type_t type,
                          const uint8_t *buffer, uint16_t size)
{
    (void)instance; (void)report_id; (void)type; (void)buffer; (void)size;
}

static void usb_task(void *argument)
{
    (void)argument;
    const tusb_rhport_init_t init = {.role = TUSB_ROLE_DEVICE, .speed = TUSB_SPEED_HIGH};
    if (!tusb_init(1, &init)) abort();
    for (;;) {
        tud_task_ext(2, false);
        p4desk_rx_stream_expire(&s_stream, esp_timer_get_time());
        atomic_store_explicit(&s_parser_errors, s_stream.parser.errors, memory_order_relaxed);
        if (atomic_exchange(&s_media_clear_requested, false)) media_clear();
        tx_service();
        media_service();
    }
}

void p4desk_usb_init(void)
{
    s_tx_release = make_queue(1);
    s_tx_priority = make_queue(12);
    s_tx_events = make_queue(4);
    s_media = xQueueCreate(8, sizeof(media_packet_t));
    uint8_t *buffer = heap_caps_malloc(P4P_MAX_JPEG, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (!s_tx_release.context || !s_tx_priority.context || !s_tx_events.context || !s_media || !buffer) abort();
    p4desk_rx_stream_init(&s_stream, buffer, P4P_MAX_JPEG, on_message, NULL);
    const usb_phy_config_t phy = {
        .controller = USB_PHY_CTRL_OTG, .target = USB_PHY_TARGET_UTMI,
        .otg_mode = USB_OTG_MODE_DEVICE, .otg_speed = USB_PHY_SPEED_HIGH,
    };
    ESP_ERROR_CHECK(usb_new_phy(&phy, &s_phy));
    if (xTaskCreatePinnedToCore(usb_task, "usb_device", 8192, NULL, 7, NULL, 0) != pdPASS) abort();
}
