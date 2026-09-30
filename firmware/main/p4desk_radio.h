#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define P4_RADIO_MAX_AP 16
#define P4_RADIO_MAX_BLE 16
#define P4_RADIO_MAX_SERVICES 12
// Internal C/Rust ABI, not a USB packet. All fields have fixed widths.
typedef struct {
    uint32_t id;
    int32_t rssi;
    uint8_t security, channel, saved, reserved;
    uint8_t name[36];
} p4_radio_ap_t;
typedef struct {
    uint32_t id;
    int32_t rssi;
    uint8_t connectable, connected, bonded, reserved;
    uint8_t name[52];
} p4_radio_ble_t;
typedef struct {
    uint32_t revision, backend;
    uint32_t wifi_on, wifi_scan, wifi_phase, wifi_error;
    uint32_t bt_on, bt_scan, bt_ready, bt_error;
    uint32_t bt_peer_id, bt_secure, phone_connected, services_count;
    uint32_t ap_count, ble_count;
    uint8_t ssid[36], ip[20], saved_ssid[36], coprocessor[20];
    p4_radio_ap_t aps[P4_RADIO_MAX_AP];
    p4_radio_ble_t devices[P4_RADIO_MAX_BLE];
    uint8_t services[P4_RADIO_MAX_SERVICES][40];
    int32_t wifi_rssi_dbm;
    uint32_t wifi_rssi_valid;
} p4_radio_snapshot_t;
_Static_assert(sizeof(p4_radio_ap_t) == 48, "radio AP ABI");
_Static_assert(sizeof(p4_radio_ble_t) == 64, "radio BLE ABI");
_Static_assert(sizeof(p4_radio_snapshot_t) == 2456, "radio state ABI");
_Static_assert(offsetof(p4_radio_snapshot_t, wifi_rssi_dbm) == 2448, "radio signal offset");
_Static_assert(offsetof(p4_radio_snapshot_t, aps) == 176, "radio AP offset");
_Static_assert(offsetof(p4_radio_snapshot_t, services) == 1968, "radio service offset");
enum {
    P4_WIFI_ENABLE = 1,
    P4_WIFI_SCAN,
    P4_WIFI_CONNECT,
    P4_WIFI_DISCONNECT,
    P4_WIFI_FORGET,
    P4_BT_ENABLE,
    P4_BT_SCAN,
    P4_BT_CONNECT,
    P4_BT_DISCONNECT,
    P4_BT_PAIR,
    P4_BT_FORGET,
    P4_WIFI_SAVED_CONNECT
};
void p4desk_radio_init(void);
bool p4desk_radio_snapshot(p4_radio_snapshot_t *out, uint32_t last_revision);
bool p4desk_radio_submit(uint32_t op, uint32_t id, const uint8_t *data, size_t length);
