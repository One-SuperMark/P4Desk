#include "p4desk_time_sync.h"
#include "p4desk_hal.h"
#include "esp_log.h"
#include "esp_netif_sntp.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"

static const char *TAG = "p4_time";
static SemaphoreHandle_t guard;
static p4_time_sync_state_t state;
static bool initialized;
static int64_t now_ms(void) { return esp_timer_get_time() / 1000; }

// ESP-IDF's documented weak override: validate BEFORE changing the shared clock.
// We own status notification, so esp_netif's wait/event callback is intentionally unused.
// This runs on lwIP's task. Never start/stop SNTP or perform I/O here.
void sntp_sync_time(struct timeval *tv) {
    if (!guard || !tv)
        return;
    xSemaphoreTake(guard, portMAX_DELAY);
    int64_t now = now_ms();
    bool accepted = p4_time_sync_accepts(&state, now);
    bool synced = false;
    if (accepted) {
        if (!p4_time_sync_valid(tv->tv_sec, tv->tv_usec))
            p4_time_sync_failure(&state, P4_TIME_INVALID, now);
        else if (!p4desk_time_set_checked((int64_t)tv->tv_sec * 1000 + tv->tv_usec / 1000))
            p4_time_sync_failure(&state, P4_TIME_APPLY, now);
        else {
            p4_time_sync_success(&state, (uint32_t)tv->tv_sec, now);
            synced = true;
        }
    }
    xSemaphoreGive(guard);
    if (synced)
        ESP_LOGI(TAG, "Wi-Fi time synchronized: unix_s=%lld", (long long)tv->tv_sec);
    else if (accepted)
        ESP_LOGW(TAG, "Wi-Fi time response rejected; current clock preserved");
}

p4_time_sync_snapshot_t p4desk_time_sync_poll(bool online, bool request) {
    if (!guard)
        guard = xSemaphoreCreateMutex();
    if (!guard)
        return (p4_time_sync_snapshot_t){.phase = P4_TIME_RETRY, .error = P4_TIME_INIT};
    xSemaphoreTake(guard, portMAX_DELAY);
    int action = p4_time_sync_step(&state, online, request, now_ms());
    p4_time_sync_snapshot_t result = state.status;
    xSemaphoreGive(guard);

    // esp_netif marshals these operations onto lwIP. Never hold guard while waiting:
    // the TCP/IP task may be finishing a reply and need it for sntp_sync_time().
    if (action == P4_TIME_STOP && initialized) {
        esp_netif_sntp_deinit();
        initialized = false;
        if (result.phase == P4_TIME_RETRY)
            ESP_LOGW(TAG, "Wi-Fi time attempt failed: reason=%lu; retry in 300 s",
                     (unsigned long)result.error);
    } else if (action == P4_TIME_START) {
        esp_sntp_config_t config = ESP_NETIF_SNTP_DEFAULT_CONFIG_MULTIPLE(
            3, ESP_SNTP_SERVER_LIST("ntp.aliyun.com", "time.cloudflare.com", "pool.ntp.org"));
        config.wait_for_sync = false;
        esp_err_t error = esp_netif_sntp_init(&config);
        initialized = error == ESP_OK;
        if (!initialized) {
            xSemaphoreTake(guard, portMAX_DELAY);
            p4_time_sync_failure(&state, P4_TIME_INIT, now_ms());
            result = state.status;
            xSemaphoreGive(guard);
            ESP_LOGW(TAG, "Wi-Fi time service unavailable: %s", esp_err_to_name(error));
        } else
            ESP_LOGI(TAG, "Wi-Fi time request started");
    }
    return result;
}
