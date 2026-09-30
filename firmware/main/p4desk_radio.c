// P4Desk radio service. The UI only copies snapshots or queues bounded commands.
// ESP-Hosted calls and NVS writes never execute on the Rust/display task.
#include "p4desk_radio.h"
#include "esp_event.h"
#include "esp_hosted.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "esp_timer.h"
#include "esp_wifi.h"
#include "freertos/FreeRTOS.h"
#include "freertos/queue.h"
#include "freertos/semphr.h"
#include "freertos/task.h"
#include "host/ble_gap.h"
#include "host/ble_gatt.h"
#include "host/ble_hs.h"
#include "host/ble_sm.h"
#include "host/util/util.h"
#include "nimble/nimble_port.h"
#include "nimble/nimble_port_freertos.h"
#include "nvs.h"
#include "nvs_flash.h"
#include "services/gap/ble_svc_gap.h"
#include "services/gatt/ble_svc_gatt.h"
#include "store/config/ble_store_config.h"
#include <stdio.h>
#include <string.h>

void ble_store_config_init(void);

static const char *TAG = "p4_radio";
static SemaphoreHandle_t guard;
static QueueHandle_t commands;
static p4_radio_snapshot_t state;
static uint32_t next_id = 1;
static bool nvs_ready, wifi_ready, wifi_started, wifi_wanted, bt_initialized, bt_synced;
static bool connecting, retry_wifi, init_failed;
static bool scan_finished_pending, save_pending, adv_pending;
static unsigned retries;
static int64_t wifi_deadline, retry_at, ble_deadline, scan_deadline, signal_sample_at;
static uint8_t own_addr_type;
static uint16_t central_handle = BLE_HS_CONN_HANDLE_NONE, phone_handle = BLE_HS_CONN_HANDLE_NONE;
static ble_addr_t addresses[P4_RADIO_MAX_BLE], central_address;
static uint32_t pending_peer_id;
static wifi_ap_record_t access_points[P4_RADIO_MAX_AP];
static wifi_config_t candidate;
static struct {
    uint32_t version;
    uint8_t ssid[32], password[64];
    uint32_t auth;
} saved;
typedef struct {
    uint32_t op, id;
    size_t length;
    uint8_t data[64];
} command_t;
enum { SAVE_PROFILE = 100, SCAN_FINISHED, RESTART_ADV };
enum {
    ERR_INIT = 1,
    ERR_SAVE,
    ERR_SCAN,
    ERR_AUTH,
    ERR_TIMEOUT,
    ERR_STALE,
    ERR_UNSUPPORTED,
    ERR_LINK,
    ERR_BLE,
    ERR_BUSY
};
static void lock(void) { xSemaphoreTake(guard, portMAX_DELAY); }
static void unlock(void) { xSemaphoreGive(guard); }
static void changed(void) {
    if (++state.revision == 0)
        ++state.revision;
}
static int64_t now_ms(void) { return esp_timer_get_time() / 1000; }
static void wipe(void *p, size_t n) {
    volatile uint8_t *s = p;
    while (n--)
        *s++ = 0;
}
static void queue_internal(uint32_t op) {
    // Events cannot be dropped when the UI command queue is full.
    lock();
    if (op == SCAN_FINISHED)
        scan_finished_pending = true;
    if (op == SAVE_PROFILE)
        save_pending = true;
    if (op == RESTART_ADV)
        adv_pending = true;
    unlock();
}
static void wifi_error(uint32_t e) {
    lock();
    state.wifi_error = e;
    changed();
    unlock();
}
static void bt_error(uint32_t e) {
    lock();
    state.bt_error = e;
    changed();
    unlock();
}
static void persist_switch(const char *key, bool enabled) {
    nvs_handle_t h;
    if (!nvs_ready || nvs_open("p4radio", NVS_READWRITE, &h) != ESP_OK) {
        if (!strcmp(key, "wifi"))
            wifi_error(ERR_SAVE);
        else
            bt_error(ERR_SAVE);
        return;
    }
    esp_err_t e = nvs_set_u8(h, key, enabled);
    if (e == ESP_OK)
        e = nvs_commit(h);
    nvs_close(h);
    if (e != ESP_OK) {
        if (!strcmp(key, "wifi"))
            wifi_error(ERR_SAVE);
        else
            bt_error(ERR_SAVE);
    }
}
static void save_profile(void) {
    nvs_handle_t h;
    if (nvs_open("p4radio", NVS_READWRITE, &h) != ESP_OK) {
        wifi_error(ERR_SAVE);
        return;
    }
    typeof(saved) value = {.version = 1, .auth = candidate.sta.threshold.authmode};
    memcpy(value.ssid, candidate.sta.ssid, 32);
    memcpy(value.password, candidate.sta.password, 64);
    esp_err_t e = nvs_set_blob(h, "profile", &value, sizeof(value));
    if (e == ESP_OK)
        e = nvs_commit(h);
    nvs_close(h);
    if (e == ESP_OK) {
        saved = value;
        lock();
        memset(state.saved_ssid, 0, sizeof(state.saved_ssid));
        memcpy(state.saved_ssid, saved.ssid, 32);
        for (unsigned i = 0; i < state.ap_count; i++)
            state.aps[i].saved = !memcmp(access_points[i].ssid, saved.ssid, 32);
        changed();
        unlock();
    } else
        wifi_error(ERR_SAVE);
    wipe(&value, sizeof(value));
}
static void wifi_event(void *arg, esp_event_base_t base, int32_t id, void *payload) {
    (void)arg;
    if (base == WIFI_EVENT && id == WIFI_EVENT_SCAN_DONE) {
        queue_internal(SCAN_FINISHED);
        return;
    }
    if (base == IP_EVENT && id == IP_EVENT_STA_GOT_IP) {
        const ip_event_got_ip_t *ip = payload;
        lock();
        if (!wifi_wanted || !state.wifi_on) {
            unlock();
            return;
        }
        state.wifi_phase = 5;
        state.wifi_rssi_valid = 0;
        signal_sample_at = 0;
        state.wifi_error = 0;
        snprintf((char *)state.ip, sizeof(state.ip), IPSTR, IP2STR(&ip->ip_info.ip));
        memcpy(state.ssid, candidate.sta.ssid, 32);
        state.ssid[32] = 0;
        connecting = false;
        retry_wifi = false;
        retries = 0;
        changed();
        unlock();
        queue_internal(SAVE_PROFILE);
    } else if (base == WIFI_EVENT && id == WIFI_EVENT_STA_DISCONNECTED) {
        const wifi_event_sta_disconnected_t *ev = payload;
        lock();
        state.ip[0] = 0;
        state.wifi_rssi_valid = 0;
        if (wifi_wanted && state.wifi_on && state.wifi_phase == 5) {
            connecting = true;
            retries = 0;
            wifi_deadline = now_ms() + 25000;
        }
        if (wifi_wanted && state.wifi_on && connecting && now_ms() < wifi_deadline && retries < 3) {
            retry_wifi = true;
            retry_at = now_ms() + 1000;
            state.wifi_phase = 4;
        } else {
            connecting = false;
            retry_wifi = false;
            state.wifi_phase = state.wifi_on ? 2 : 0;
            if (wifi_wanted)
                state.wifi_error =
                    (ev->reason == WIFI_REASON_AUTH_FAIL || ev->reason == WIFI_REASON_4WAY_HANDSHAKE_TIMEOUT)
                        ? ERR_AUTH
                        : ERR_LINK;
        }
        changed();
        unlock();
    }
}
static bool ensure_wifi(void) {
    if (wifi_ready)
        return true;
    // A partially initialized transport is not safe to initialize a second time.
    if (init_failed)
        return false;
    lock();
    state.backend = 1;
    changed();
    unlock();
    if (!nvs_ready)
        goto fail;
    esp_err_t e = esp_netif_init();
    if (e != ESP_OK && e != ESP_ERR_INVALID_STATE)
        goto fail;
    e = esp_event_loop_create_default();
    if (e != ESP_OK && e != ESP_ERR_INVALID_STATE)
        goto fail;
    if (!esp_netif_create_default_wifi_sta())
        goto fail;
    if (esp_event_handler_register(WIFI_EVENT, ESP_EVENT_ANY_ID, wifi_event, NULL) != ESP_OK ||
        esp_event_handler_register(IP_EVENT, IP_EVENT_STA_GOT_IP, wifi_event, NULL) != ESP_OK)
        goto fail;
    // Defer Hosted's upstream constructor until after the TF slot is mounted.
    if (esp_hosted_init() != ESP_OK)
        goto fail;
    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    if (esp_wifi_init(&cfg) != ESP_OK)
        goto fail;
    if (esp_wifi_set_storage(WIFI_STORAGE_RAM) != ESP_OK || esp_wifi_set_mode(WIFI_MODE_STA) != ESP_OK)
        goto fail;
    wifi_ready = true;
    lock();
    state.backend = 2;
    changed();
    unlock();
    // Version is diagnostic only: older factory firmware can omit this RPC.
    ESP_LOGI(TAG, "radio transport initialized");
    return true;
fail:
    init_failed = true;
    lock();
    state.backend = 3;
    state.wifi_error = ERR_INIT;
    state.bt_error = ERR_INIT;
    changed();
    unlock();
    ESP_LOGW(TAG, "radio initialization failed");
    return false;
}
static bool start_wifi(void) {
    if (!ensure_wifi())
        return false;
    if (!wifi_started) {
        if (esp_wifi_start() != ESP_OK) {
            wifi_error(ERR_INIT);
            return false;
        }
        wifi_started = true;
    }
    return true;
}
static void scan_wifi(void) {
    lock();
    bool allowed = state.wifi_on && !state.wifi_scan && !connecting;
    unlock();
    if (!allowed)
        return;
    if (!start_wifi())
        return;
    wifi_scan_config_t config = {.show_hidden = false, .scan_type = WIFI_SCAN_TYPE_ACTIVE};
    lock();
    state.wifi_scan = 1;
    scan_finished_pending = false;
    scan_deadline = now_ms() + 15000;
    state.wifi_error = 0;
    changed();
    unlock();
    if (esp_wifi_scan_start(&config, false) != ESP_OK) {
        lock();
        state.wifi_scan = 0;
        state.wifi_error = ERR_SCAN;
        changed();
        unlock();
    }
}
static void finish_scan(void) {
    lock();
    bool valid = state.wifi_on && state.wifi_scan;
    unlock();
    if (!valid)
        return;
    wifi_ap_record_t records[32];
    uint16_t count = 32;
    esp_err_t e = esp_wifi_scan_get_ap_records(&count, records);
    lock();
    state.wifi_scan = 0;
    if (e != ESP_OK) {
        state.wifi_error = ERR_SCAN;
        changed();
        unlock();
        return;
    }
    state.ap_count = 0;
    for (unsigned i = 0; i < count && state.ap_count < P4_RADIO_MAX_AP; i++) {
        if (!records[i].ssid[0])
            continue;
        bool duplicate = false;
        for (unsigned j = 0; j < state.ap_count; j++)
            if (!memcmp(access_points[j].ssid, records[i].ssid, 32) &&
                access_points[j].authmode == records[i].authmode)
                duplicate = true;
        if (duplicate)
            continue;
        unsigned n = state.ap_count++;
        access_points[n] = records[i];
        p4_radio_ap_t *a = &state.aps[n];
        memset(a, 0, sizeof(*a));
        a->id = next_id++;
        a->rssi = records[i].rssi;
        a->channel = records[i].primary;
        a->security =
            records[i].authmode == WIFI_AUTH_OPEN ? 0
            : (records[i].authmode == WIFI_AUTH_WPA2_PSK || records[i].authmode == WIFI_AUTH_WPA_WPA2_PSK ||
               records[i].authmode == WIFI_AUTH_WPA3_PSK || records[i].authmode == WIFI_AUTH_WPA2_WPA3_PSK)
                ? 1
                : 2;
        a->saved = saved.version == 1 && !memcmp(saved.ssid, records[i].ssid, 32);
        memcpy(a->name, records[i].ssid, 32);
    }
    unsigned found = state.ap_count;
    changed();
    unlock();
    ESP_LOGI(TAG, "Wi-Fi scan complete: networks=%u", found);
}
static void connect_wifi(const command_t *c) {
    lock();
    bool on = state.wifi_on;
    unlock();
    if (!on || !start_wifi())
        return;
    wifi_config_t config = {0};
    if (c->op == P4_WIFI_SAVED_CONNECT) {
        if (saved.version != 1) {
            wifi_error(ERR_STALE);
            return;
        }
        memcpy(config.sta.ssid, saved.ssid, 32);
        memcpy(config.sta.password, saved.password, 64);
        config.sta.threshold.authmode = saved.auth;
    } else {
        lock();
        int index = -1;
        for (unsigned i = 0; i < state.ap_count; i++)
            if (state.aps[i].id == c->id)
                index = i;
        if (index < 0) {
            state.wifi_error = ERR_STALE;
            changed();
            unlock();
            return;
        }
        uint8_t security = state.aps[index].security;
        memcpy(config.sta.ssid, access_points[index].ssid, 32);
        config.sta.threshold.authmode = security ? WIFI_AUTH_WPA2_PSK : WIFI_AUTH_OPEN;
        unlock();
        if (security == 2) {
            wifi_error(ERR_UNSUPPORTED);
            return;
        }
        if (security && (c->length < 8 || c->length > 63)) {
            wifi_error(ERR_AUTH);
            return;
        }
        if (!security && c->length) {
            wifi_error(ERR_AUTH);
            return;
        }
        memcpy(config.sta.password, c->data, c->length);
    }
    config.sta.pmf_cfg.capable = true;
    config.sta.pmf_cfg.required = false;
    config.sta.sae_pwe_h2e = WPA3_SAE_PWE_BOTH;
    lock();
    wifi_wanted = false;
    connecting = false;
    retry_wifi = false;
    unlock();
    esp_wifi_scan_stop();
    esp_wifi_disconnect();
    // Let the old disconnect event drain before arming the new attempt.
    vTaskDelay(pdMS_TO_TICKS(100));
    lock();
    candidate = config;
    save_pending = false;
    unlock();
    esp_err_t e = esp_wifi_set_config(WIFI_IF_STA, &config);
    wipe(&config, sizeof(config));
    if (e != ESP_OK) {
        wifi_error(ERR_INIT);
        return;
    }
    lock();
    wifi_wanted = true;
    connecting = true;
    retries = 0;
    wifi_deadline = now_ms() + 25000;
    state.wifi_phase = 4;
    state.wifi_rssi_valid = 0;
    state.wifi_error = 0;
    state.wifi_scan = 0;
    state.ip[0] = 0;
    memcpy(state.ssid, candidate.sta.ssid, 32);
    state.ssid[32] = 0;
    changed();
    unlock();
    if (esp_wifi_connect() != ESP_OK) {
        lock();
        connecting = false;
        state.wifi_phase = 2;
        state.wifi_error = ERR_LINK;
        changed();
        unlock();
    }
}

static int gap_event(struct ble_gap_event *event, void *arg);
static int read_info(uint16_t connection, uint16_t attr, struct ble_gatt_access_ctxt *ctx, void *arg) {
    (void)connection;
    (void)attr;
    const char *value = arg;
    return os_mbuf_append(ctx->om, value, strlen(value)) == 0 ? 0 : BLE_ATT_ERR_INSUFFICIENT_RES;
}
static const struct ble_gatt_svc_def services[] = {
    {.type = BLE_GATT_SVC_TYPE_PRIMARY,
     .uuid = BLE_UUID16_DECLARE(0x180a),
     .characteristics = (struct ble_gatt_chr_def[]){{.uuid = BLE_UUID16_DECLARE(0x2a29),
                                                     .access_cb = read_info,
                                                     .arg = "P4Desk",
                                                     .flags = BLE_GATT_CHR_F_READ},
                                                    {.uuid = BLE_UUID16_DECLARE(0x2a24),
                                                     .access_cb = read_info,
                                                     .arg = "ESP32-P4 7B",
                                                     .flags = BLE_GATT_CHR_F_READ},
                                                    {.uuid = BLE_UUID16_DECLARE(0x2a26),
                                                     .access_cb = read_info,
                                                     .arg = "0.1.0",
                                                     .flags = BLE_GATT_CHR_F_READ},
                                                    {0}}},
    {0}};
static void advertise(void) {
    lock();
    bool enabled = state.bt_on && state.bt_ready && phone_handle == BLE_HS_CONN_HANDLE_NONE;
    unlock();
    if (!enabled || ble_gap_adv_active())
        return;
    struct ble_hs_adv_fields fields = {0};
    fields.flags = BLE_HS_ADV_F_DISC_GEN | BLE_HS_ADV_F_BREDR_UNSUP;
    fields.name = (uint8_t *)"P4Desk";
    fields.name_len = 6;
    fields.name_is_complete = 1;
    ble_uuid16_t uuid = BLE_UUID16_INIT(0x180a);
    fields.uuids16 = &uuid;
    fields.num_uuids16 = 1;
    fields.uuids16_is_complete = 1;
    struct ble_gap_adv_params params = {.conn_mode = BLE_GAP_CONN_MODE_UND,
                                        .disc_mode = BLE_GAP_DISC_MODE_GEN};
    if (ble_gap_adv_set_fields(&fields) ||
        ble_gap_adv_start(own_addr_type, NULL, BLE_HS_FOREVER, &params, gap_event, NULL))
        bt_error(ERR_BLE);
}
static int service_found(uint16_t conn, const struct ble_gatt_error *error, const struct ble_gatt_svc *svc,
                         void *arg) {
    (void)arg;
    lock();
    if (conn == central_handle && error->status == 0 && state.services_count < P4_RADIO_MAX_SERVICES) {
        ble_uuid_to_str(&svc->uuid.u, (char *)state.services[state.services_count++]);
        changed();
    } else if (conn == central_handle && error->status != 0 && error->status != BLE_HS_EDONE) {
        state.bt_error = ERR_BLE;
        changed();
    }
    unlock();
    return 0;
}
static int gap_event(struct ble_gap_event *event, void *arg) {
    (void)arg;
    switch (event->type) {
    case BLE_GAP_EVENT_DISC: {
        struct ble_hs_adv_fields fields;
        memset(&fields, 0, sizeof(fields));
        if (ble_hs_adv_parse_fields(&fields, event->disc.data, event->disc.length_data))
            break;
        lock();
        if (!state.bt_on || !state.bt_scan) {
            unlock();
            break;
        }
        int i = -1;
        bool dirty = false;
        for (unsigned j = 0; j < state.ble_count; j++)
            if (!memcmp(&addresses[j], &event->disc.addr, sizeof(ble_addr_t)))
                i = j;
        if (i < 0 && state.ble_count < P4_RADIO_MAX_BLE) {
            i = state.ble_count++;
            dirty = true;
            addresses[i] = event->disc.addr;
            memset(&state.devices[i], 0, sizeof(state.devices[i]));
            state.devices[i].id = next_id++;
        }
        if (i >= 0) {
            p4_radio_ble_t *d = &state.devices[i];
            dirty |= d->rssi != event->disc.rssi;
            d->rssi = event->disc.rssi;
            if (event->disc.event_type == BLE_HCI_ADV_RPT_EVTYPE_ADV_IND ||
                event->disc.event_type == BLE_HCI_ADV_RPT_EVTYPE_DIR_IND) {
                dirty |= !d->connectable;
                d->connectable = 1;
            }
            if (fields.name && fields.name_len) {
                size_t n = fields.name_len < sizeof(d->name) - 1 ? fields.name_len : sizeof(d->name) - 1;
                if (memcmp(d->name, fields.name, n))
                    dirty = true;
                memset(d->name, 0, sizeof(d->name));
                memcpy(d->name, fields.name, n);
            }
            if (dirty)
                changed();
        }
        unlock();
        break;
    }
    case BLE_GAP_EVENT_DISC_COMPLETE: {
        lock();
        state.bt_scan = 0;
        unsigned found = state.ble_count;
        changed();
        unlock();
        ESP_LOGI(TAG, "BLE scan complete: devices=%u", found);
        break;
    }
    case BLE_GAP_EVENT_CONNECT: {
        if (event->connect.status) {
            lock();
            state.bt_error = ERR_LINK;
            pending_peer_id = 0;
            state.bt_peer_id = 0;
            changed();
            unlock();
            queue_internal(RESTART_ADV);
            break;
        }
        struct ble_gap_conn_desc desc;
        if (ble_gap_conn_find(event->connect.conn_handle, &desc))
            break;
        lock();
        bool enabled = state.bt_on;
        if (desc.role == BLE_GAP_ROLE_MASTER) {
            central_handle = event->connect.conn_handle;
            central_address = desc.peer_id_addr;
            state.bt_peer_id = pending_peer_id;
            state.bt_secure = desc.sec_state.encrypted;
            pending_peer_id = 0;
            for (unsigned i = 0; i < state.ble_count; i++)
                state.devices[i].connected = state.devices[i].id == state.bt_peer_id;
            state.services_count = 0;
            memset(state.services, 0, sizeof(state.services));
        } else {
            phone_handle = event->connect.conn_handle;
            state.phone_connected = 1;
        }
        state.bt_error = 0;
        changed();
        unlock();
        if (!enabled)
            ble_gap_terminate(event->connect.conn_handle, BLE_ERR_REM_USER_CONN_TERM);
        else if (desc.role == BLE_GAP_ROLE_MASTER)
            ble_gattc_disc_all_svcs(event->connect.conn_handle, service_found, NULL);
        break;
    }
    case BLE_GAP_EVENT_DISCONNECT:
        lock();
        if (event->disconnect.conn.conn_handle == central_handle) {
            central_handle = BLE_HS_CONN_HANDLE_NONE;
            state.bt_peer_id = 0;
            state.bt_secure = 0;
            state.services_count = 0;
            for (unsigned i = 0; i < state.ble_count; i++)
                state.devices[i].connected = 0;
        }
        if (event->disconnect.conn.conn_handle == phone_handle) {
            phone_handle = BLE_HS_CONN_HANDLE_NONE;
            state.phone_connected = 0;
        }
        changed();
        unlock();
        queue_internal(RESTART_ADV);
        break;
    case BLE_GAP_EVENT_ENC_CHANGE: {
        struct ble_gap_conn_desc desc;
        if (ble_gap_conn_find(event->enc_change.conn_handle, &desc))
            break;
        lock();
        if (event->enc_change.conn_handle == central_handle) {
            state.bt_secure = desc.sec_state.encrypted;
            for (unsigned i = 0; i < state.ble_count; i++)
                if (state.devices[i].id == state.bt_peer_id)
                    state.devices[i].bonded = desc.sec_state.bonded;
        }
        if (event->enc_change.status)
            state.bt_error = ERR_AUTH;
        changed();
        unlock();
        break;
    }
    case BLE_GAP_EVENT_REPEAT_PAIRING:
        return BLE_GAP_REPEAT_PAIRING_IGNORE;
    default:
        break;
    }
    return 0;
}
static void scan_ble(void);
static void ble_sync(void) {
    if (ble_hs_util_ensure_addr(0) || ble_hs_id_infer_auto(0, &own_addr_type)) {
        bt_error(ERR_INIT);
        return;
    }
    lock();
    bt_synced = true;
    state.bt_ready = 1;
    state.bt_error = 0;
    changed();
    unlock();
    queue_internal(RESTART_ADV);
    // Discover peers after the controller has actually synchronized.
    command_t cmd = {.op = P4_BT_SCAN};
    xQueueSend(commands, &cmd, 0);
    ESP_LOGI(TAG, "BLE controller synchronized");
}
static void ble_reset(int reason) {
    (void)reason;
    lock();
    bt_synced = false;
    state.bt_ready = 0;
    state.bt_scan = 0;
    state.bt_peer_id = 0;
    state.phone_connected = 0;
    central_handle = phone_handle = BLE_HS_CONN_HANDLE_NONE;
    pending_peer_id = 0;
    state.services_count = 0;
    state.bt_secure = 0;
    for (unsigned i = 0; i < state.ble_count; i++)
        state.devices[i].connected = 0;
    ble_deadline = now_ms() + 12000;
    state.bt_error = ERR_LINK;
    changed();
    unlock();
}
static void ble_task(void *arg) {
    (void)arg;
    nimble_port_run();
    nimble_port_freertos_deinit();
}
static void enable_ble(bool on) {
    lock();
    state.bt_on = on;
    state.bt_error = 0;
    changed();
    unlock();
    persist_switch("ble", on);
    if (!on) {
        if (bt_synced) {
            ble_gap_disc_cancel();
            ble_gap_adv_stop();
            ble_gap_conn_cancel();
            if (central_handle != BLE_HS_CONN_HANDLE_NONE)
                ble_gap_terminate(central_handle, BLE_ERR_REM_USER_CONN_TERM);
            if (phone_handle != BLE_HS_CONN_HANDLE_NONE)
                ble_gap_terminate(phone_handle, BLE_ERR_REM_USER_CONN_TERM);
        }
        lock();
        state.bt_scan = 0;
        pending_peer_id = 0;
        state.bt_peer_id = 0;
        changed();
        unlock();
        return;
    }
    if (!ensure_wifi())
        return;
    if (!bt_initialized) {
        if (nimble_port_init() != ESP_OK) {
            bt_error(ERR_INIT);
            return;
        }
        bt_initialized = true;
        ble_hs_cfg.sync_cb = ble_sync;
        ble_hs_cfg.reset_cb = ble_reset;
        ble_hs_cfg.store_status_cb = ble_store_util_status_rr;
        ble_hs_cfg.sm_io_cap = BLE_HS_IO_NO_INPUT_OUTPUT;
        ble_hs_cfg.sm_bonding = 1;
        ble_hs_cfg.sm_sc = 1;
        ble_hs_cfg.sm_our_key_dist = BLE_SM_PAIR_KEY_DIST_ENC | BLE_SM_PAIR_KEY_DIST_ID;
        ble_hs_cfg.sm_their_key_dist = BLE_SM_PAIR_KEY_DIST_ENC | BLE_SM_PAIR_KEY_DIST_ID;
        ble_svc_gap_init();
        ble_svc_gatt_init();
        if (ble_gatts_count_cfg(services) || ble_gatts_add_svcs(services)) {
            bt_error(ERR_INIT);
            return;
        }
        ble_svc_gap_device_name_set("P4Desk");
        ble_store_config_init();
        ble_deadline = now_ms() + 12000;
        nimble_port_freertos_init(ble_task);
    } else
        advertise();
}
static void scan_ble(void) {
    lock();
    bool allowed = state.bt_on && bt_synced && !state.bt_scan && !pending_peer_id;
    unlock();
    if (!allowed)
        return;
    lock();
    // Keep connected peers and their IDs stable when rescanning.
    for (unsigned i = 0; i < state.ble_count;) {
        if (state.devices[i].connected) {
            i++;
            continue;
        }
        unsigned n = --state.ble_count;
        state.devices[i] = state.devices[n];
        addresses[i] = addresses[n];
    }
    state.bt_scan = 1;
    state.bt_error = 0;
    changed();
    unlock();
    struct ble_gap_disc_params params = {.passive = 0, .itvl = 160, .window = 48, .filter_duplicates = 0};
    if (ble_gap_disc(own_addr_type, 10000, &params, gap_event, NULL)) {
        lock();
        state.bt_scan = 0;
        state.bt_error = ERR_SCAN;
        changed();
        unlock();
    }
}
static void connect_ble(uint32_t id) {
    lock();
    int index = -1;
    for (unsigned i = 0; i < state.ble_count; i++)
        if (state.devices[i].id == id && state.devices[i].connectable)
            index = i;
    bool allowed = state.bt_on && bt_synced && central_handle == BLE_HS_CONN_HANDLE_NONE && !pending_peer_id;
    if (index < 0 || !allowed) {
        state.bt_error = index < 0 ? ERR_STALE : ERR_BUSY;
        changed();
        unlock();
        return;
    }
    ble_addr_t addr = addresses[index];
    pending_peer_id = id;
    state.bt_peer_id = id;
    state.bt_error = 0;
    changed();
    unlock();
    ble_gap_disc_cancel();
    if (ble_gap_connect(own_addr_type, &addr, 10000, NULL, gap_event, NULL)) {
        lock();
        pending_peer_id = 0;
        state.bt_peer_id = 0;
        state.bt_error = ERR_LINK;
        changed();
        unlock();
    }
}
static void process(const command_t *c) {
    switch (c->op) {
    case P4_WIFI_ENABLE:
        lock();
        state.wifi_on = c->id != 0;
        state.wifi_error = 0;
        changed();
        unlock();
        persist_switch("wifi", c->id != 0);
        if (c->id) {
            if (start_wifi()) {
                lock();
                state.wifi_phase = 2;
                changed();
                unlock();
                if (saved.version == 1) {
                    command_t s = {.op = P4_WIFI_SAVED_CONNECT};
                    connect_wifi(&s);
                } else
                    scan_wifi();
            }
        } else {
            lock();
            wifi_wanted = false;
            connecting = false;
            retry_wifi = false;
            unlock();
            if (wifi_started) {
                esp_wifi_scan_stop();
                esp_wifi_disconnect();
                esp_wifi_stop();
                wifi_started = false;
            }
            lock();
            state.wifi_phase = 0;
            state.wifi_rssi_valid = 0;
            state.wifi_scan = 0;
            state.ip[0] = 0;
            state.ssid[0] = 0;
            changed();
            unlock();
        }
        break;
    case P4_WIFI_SCAN:
        scan_wifi();
        break;
    case SCAN_FINISHED:
        finish_scan();
        break;
    case P4_WIFI_CONNECT:
    case P4_WIFI_SAVED_CONNECT:
        connect_wifi(c);
        break;
    case SAVE_PROFILE:
        lock();
        bool connected = state.wifi_phase == 5;
        unlock();
        if (connected)
            save_profile();
        break;
    case P4_WIFI_DISCONNECT:
        lock();
        wifi_wanted = false;
        connecting = false;
        retry_wifi = false;
        state.wifi_phase = state.wifi_on ? 2 : 0;
        state.ip[0] = 0;
        changed();
        unlock();
        if (wifi_ready)
            esp_wifi_disconnect();
        break;
    case P4_WIFI_FORGET: {
        nvs_handle_t h;
        if (nvs_open("p4radio", NVS_READWRITE, &h) != ESP_OK) {
            wifi_error(ERR_SAVE);
            break;
        }
        esp_err_t e = nvs_erase_key(h, "profile");
        if (e == ESP_ERR_NVS_NOT_FOUND)
            e = ESP_OK;
        if (e == ESP_OK)
            e = nvs_commit(h);
        nvs_close(h);
        if (e != ESP_OK) {
            wifi_error(ERR_SAVE);
            break;
        }
        wipe(&saved, sizeof(saved));
        lock();
        state.saved_ssid[0] = 0;
        for (unsigned i = 0; i < state.ap_count; i++)
            state.aps[i].saved = 0;
        changed();
        unlock();
        command_t d = {.op = P4_WIFI_DISCONNECT};
        process(&d);
        wipe(&candidate, sizeof(candidate));
        break;
    }
    case P4_BT_ENABLE:
        enable_ble(c->id != 0);
        break;
    case P4_BT_SCAN:
        scan_ble();
        break;
    case P4_BT_CONNECT:
        connect_ble(c->id);
        break;
    case P4_BT_DISCONNECT:
        if (central_handle != BLE_HS_CONN_HANDLE_NONE)
            ble_gap_terminate(central_handle, BLE_ERR_REM_USER_CONN_TERM);
        if (phone_handle != BLE_HS_CONN_HANDLE_NONE)
            ble_gap_terminate(phone_handle, BLE_ERR_REM_USER_CONN_TERM);
        break;
    case P4_BT_PAIR:
        if (central_handle != BLE_HS_CONN_HANDLE_NONE && ble_gap_security_initiate(central_handle))
            bt_error(ERR_AUTH);
        break;
    case P4_BT_FORGET:
        if (central_handle != BLE_HS_CONN_HANDLE_NONE) {
            if (ble_store_util_delete_peer(&central_address))
                bt_error(ERR_SAVE);
            else
                ble_gap_terminate(central_handle, BLE_ERR_REM_USER_CONN_TERM);
        }
        break;
    case RESTART_ADV:
        advertise();
        break;
    }
}
// Poll the current association instead of using a stale scan-list RSSI.
// RPC stays on this worker, never on the UI/display task.
static void sample_wifi_signal(void) {
    lock();
    bool due = state.wifi_on && state.wifi_phase == 5 && now_ms() >= signal_sample_at;
    if (due)
        signal_sample_at = now_ms() + 5000;
    unlock();
    if (!due)
        return;
    wifi_ap_record_t ap = {0};
    esp_err_t error = esp_wifi_sta_get_ap_info(&ap);
    lock();
    bool connected = state.wifi_on && state.wifi_phase == 5;
    bool valid =
        connected && error == ESP_OK && ap.rssi < 0 && ap.rssi >= -127 && !memcmp(ap.ssid, state.ssid, 32);
    int32_t dbm = valid ? ap.rssi : 0;
    bool first = valid && !state.wifi_rssi_valid;
    if (state.wifi_rssi_valid != valid || state.wifi_rssi_dbm != dbm) {
        state.wifi_rssi_valid = valid;
        state.wifi_rssi_dbm = dbm;
        changed();
    }
    unlock();
    if (first)
        ESP_LOGI(TAG, "Wi-Fi signal sampling ready: RSSI=%ld dBm", (long)dbm);
}
static void worker(void *arg) {
    (void)arg;
    esp_err_t e = nvs_flash_init(); // Never erase a user's NVS to recover an init error.
    nvs_ready = e == ESP_OK;
    if (nvs_ready) {
        nvs_handle_t h;
        uint8_t wifi = 1, ble = 1;
        if (nvs_open("p4radio", NVS_READONLY, &h) == ESP_OK) {
            size_t size = sizeof(saved);
            if (nvs_get_blob(h, "profile", &saved, &size) != ESP_OK || size != sizeof(saved) ||
                saved.version != 1)
                wipe(&saved, sizeof(saved));
            nvs_get_u8(h, "wifi", &wifi);
            nvs_get_u8(h, "ble", &ble);
            nvs_close(h);
            lock();
            memcpy(state.saved_ssid, saved.ssid, 32);
            changed();
            unlock();
        }
        if (wifi) {
            command_t c = {.op = P4_WIFI_ENABLE, .id = 1};
            process(&c);
        }
        if (ble)
            enable_ble(true);
    } else {
        lock();
        state.backend = 3;
        state.wifi_error = state.bt_error = ERR_SAVE;
        changed();
        unlock();
    }
    while (true) {
        command_t c;
        if (xQueueReceive(commands, &c, pdMS_TO_TICKS(100)) == pdTRUE) {
            process(&c);
            wipe(&c, sizeof(c));
        }
        lock();
        bool scan_done = scan_finished_pending, save = save_pending, adv = adv_pending;
        scan_finished_pending = save_pending = adv_pending = false;
        unlock();
        if (scan_done)
            finish_scan();
        if (save) {
            command_t internal = {.op = SAVE_PROFILE};
            process(&internal);
        }
        if (adv)
            advertise();
        lock();
        int64_t now = now_ms();
        bool scan_timeout = state.wifi_scan && now >= scan_deadline;
        if (scan_timeout) {
            state.wifi_scan = 0;
            state.wifi_error = ERR_TIMEOUT;
            changed();
        }
        bool retry = retry_wifi && wifi_wanted && now >= retry_at && now < wifi_deadline;
        if (retry) {
            retry_wifi = false;
            retries++;
        }
        bool timeout = connecting && now >= wifi_deadline;
        if (timeout) {
            connecting = false;
            retry_wifi = false;
            wifi_wanted = false;
            state.wifi_phase = 2;
            state.wifi_error = ERR_TIMEOUT;
            changed();
        }
        if (state.bt_on && bt_initialized && !bt_synced && now >= ble_deadline &&
            state.bt_error != ERR_TIMEOUT) {
            state.bt_error = ERR_TIMEOUT;
            changed();
        }
        unlock();
        if (scan_timeout)
            esp_wifi_scan_stop();
        if (timeout)
            esp_wifi_disconnect();
        if (retry && esp_wifi_connect() != ESP_OK)
            wifi_error(ERR_LINK);
        sample_wifi_signal();
    }
}
void p4desk_radio_init(void) {
    if (guard)
        return;
    guard = xSemaphoreCreateMutex();
    commands = xQueueCreate(8, sizeof(command_t));
    if (!guard || !commands) {
        ESP_LOGE(TAG, "radio allocation failed");
        return;
    }
    state.revision = 1;
    // Third-party network logs can include network/device identifiers.
    const char *tags[] = {"wifi",   "wifi_init", "rpc_evt",   "rpc_rsp",       "rpc_wrap",
                          "NimBLE", "ble_hs",    "transport", "transport_drv", "sdio_wrapper"};
    for (unsigned i = 0; i < sizeof(tags) / sizeof(tags[0]); i++)
        esp_log_level_set(tags[i], ESP_LOG_WARN);
    if (xTaskCreate(worker, "p4_radio", 8192, NULL, 4, NULL) != pdPASS) {
        state.backend = 3;
        state.wifi_error = state.bt_error = ERR_INIT;
    }
}
bool p4desk_radio_snapshot(p4_radio_snapshot_t *out, uint32_t last_revision) {
    if (!guard || !out)
        return false;
    lock();
    bool updated = state.revision != last_revision;
    if (updated)
        *out = state;
    unlock();
    return updated;
}
bool p4desk_radio_submit(uint32_t op, uint32_t id, const uint8_t *data, size_t length) {
    if (!commands || op < P4_WIFI_ENABLE || op > P4_WIFI_SAVED_CONNECT || length > 63 || (length && !data))
        return false;
    if (op != P4_WIFI_CONNECT && length)
        return false;
    command_t c = {.op = op, .id = id, .length = length};
    if (length)
        memcpy(c.data, data, length);
    bool ok = xQueueSend(commands, &c, 0) == pdTRUE;
    wipe(&c, sizeof(c));
    return ok;
}
