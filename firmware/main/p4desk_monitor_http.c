// Bounded HTTPS transport for the Rust monitor worker. No display ownership.
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include "esp_http_client.h"
#include "esp_crt_bundle.h"
#include "esp_timer.h"
#include "esp_log.h"
#include "esp_heap_caps.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "monitor_http_body.h"

#define MONITOR_MAX_KEY 512
#define MONITOR_MAX_AUTHORITY 240
#define MONITOR_MAX_URL 4096
#define MONITOR_MAX_BODY 16384
#define MONITOR_REQUEST_US 12000000
#define MONITOR_IO_TIMEOUT_MS 6000

// One Rust worker owns this handle. It calls reset at every batch boundary,
// including cancellation/error, so TLS memory is not retained while idle.
static struct {
    esp_http_client_handle_t client;
    char authority[MONITOR_MAX_AUTHORITY + 1];
    char key[MONITOR_MAX_KEY + 1];
} session;

static void clear_secret(char *value, size_t length)
{
    volatile char *p = value;
    while (length--) *p++ = 0;
}

void p4desk_monitor_http_reset(void)
{
    if (session.client) {
        char *header_key = NULL;
        if (esp_http_client_get_header(session.client, "x-api-key", &header_key) == ESP_OK && header_key) {
            clear_secret(header_key, strlen(header_key));
        }
        esp_http_client_cleanup(session.client); // Also closes the transport.
        session.client = NULL;
    }
    clear_secret(session.key, sizeof(session.key));
    memset(session.authority, 0, sizeof(session.authority));
}

// Compare the exact HTTPS authority, including an explicit port. Conservative
// comparisons reconnect for equivalent spellings rather than sharing a session
// across origins. Rust also validates the configured site before reaching FFI.
static bool request_identity(const char *url, const char *key, size_t *authority_length)
{
    if (!url || !key || strncmp(url, "https://", 8) != 0) return false;
    size_t url_length = strnlen(url, MONITOR_MAX_URL + 1);
    size_t key_length = strnlen(key, MONITOR_MAX_KEY + 1);
    if (url_length > MONITOR_MAX_URL || !key_length || key_length > MONITOR_MAX_KEY) return false;
    for (size_t i = 0; i < url_length; i++) {
        unsigned char ch = (unsigned char)url[i];
        if (ch < 33 || ch > 126 || ch == '#' || ch == '\\') return false;
    }
    for (size_t i = 0; i < key_length; i++) {
        unsigned char ch = (unsigned char)key[i];
        if (ch < 33 || ch > 126) return false;
    }
    size_t length = 8;
    while (url[length] && url[length] != '/' && url[length] != '?') {
        if (url[length] == '@') return false;
        length++;
    }
    if (length == 8 || length > MONITOR_MAX_AUTHORITY) return false;
    *authority_length = length;
    return true;
}

static bool set_remaining_timeout(esp_http_client_handle_t client, int64_t deadline)
{
    int64_t remaining = deadline - esp_timer_get_time();
    if (remaining <= 0) return false;
    int timeout_ms = (int)((remaining + 999) / 1000);
    if (timeout_ms > MONITOR_IO_TIMEOUT_MS) timeout_ms = MONITOR_IO_TIMEOUT_MS;
    return esp_http_client_set_timeout_ms(client, timeout_ms) == ESP_OK;
}

typedef struct {
    esp_http_client_handle_t client;
    int64_t deadline;
} body_reader_t;

static int read_body(void *context, uint8_t *out, size_t count)
{
    body_reader_t *reader = context;
    if (!set_remaining_timeout(reader->client, reader->deadline)) return -1;
    // Bound each synchronous SDK read as well as the final response buffer.
    if (count > 2048) count = 2048;
    int n = esp_http_client_read(reader->client, (char *)out, (int)count);
    return esp_timer_get_time() >= reader->deadline ? -1 : n;
}

static bool body_complete(void *context)
{
    return esp_http_client_is_complete_data_received(((body_reader_t *)context)->client);
}

// HTTP status, or -1 transport, -2 capacity, -3 incomplete body.
int32_t p4desk_monitor_http(const char *url, const char *key, const char *body,
                            uint8_t *out, size_t capacity, size_t *length) {
    if (length) *length = 0;
    size_t authority_length = 0;
    size_t body_length = body ? strnlen(body, MONITOR_MAX_BODY + 1) : 0;
    if (!out || !length || capacity > 262144 || body_length > MONITOR_MAX_BODY ||
        !request_identity(url, key, &authority_length)) {
        p4desk_monitor_http_reset();
        return -1;
    }
    const int64_t began = esp_timer_get_time();
    const int64_t deadline = began + MONITOR_REQUEST_US;
    // Library error logs can include URLs. Never log credentials or responses.
    esp_log_level_set("HTTP_CLIENT", ESP_LOG_NONE);
    bool reused = session.client && strlen(session.authority) == authority_length &&
                  memcmp(session.authority, url, authority_length) == 0 && strcmp(session.key, key) == 0;
    if (!reused) p4desk_monitor_http_reset();
    ESP_LOGI("p4desk_monitor", "https begin reused=%u internal_free=%zu dma_free=%zu",
             (unsigned)reused,
             heap_caps_get_free_size(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT),
             heap_caps_get_free_size(MALLOC_CAP_INTERNAL | MALLOC_CAP_DMA));
    int32_t result = -1;
    if (!session.client) {
        const esp_http_client_config_t config = {
            .url = url, .method = body ? HTTP_METHOD_POST : HTTP_METHOD_GET,
            .timeout_ms = MONITOR_IO_TIMEOUT_MS, .disable_auto_redirect = true,
            .crt_bundle_attach = esp_crt_bundle_attach,
            .buffer_size = 2048, .buffer_size_tx = 2048,
            .keep_alive_enable = true,
        };
        session.client = esp_http_client_init(&config);
        if (!session.client) goto done;
        memcpy(session.authority, url, authority_length);
        session.authority[authority_length] = 0;
        memcpy(session.key, key, strlen(key) + 1);
    }
    esp_http_client_handle_t client = session.client;
    if (esp_http_client_set_url(client, url) != ESP_OK ||
        esp_http_client_set_method(client, body ? HTTP_METHOD_POST : HTTP_METHOD_GET) != ESP_OK ||
        esp_http_client_set_header(client, "x-api-key", key) != ESP_OK ||
        esp_http_client_set_header(client, "Accept", "application/json") != ESP_OK ||
        esp_http_client_set_header(client, "Accept-Encoding", "identity") != ESP_OK ||
        esp_http_client_set_header(client, "Content-Type", "application/json") != ESP_OK ||
        !set_remaining_timeout(client, deadline) ||
        esp_http_client_open(client, (int)body_length) != ESP_OK ||
        esp_timer_get_time() >= deadline) goto done;
    for (int sent = 0; sent < (int)body_length;) {
        if (!set_remaining_timeout(client, deadline)) goto done;
        int n = esp_http_client_write(client, body + sent, (int)body_length - sent);
        if (n <= 0 || esp_timer_get_time() > deadline) goto done;
        sent += n;
    }
    if (!set_remaining_timeout(client, deadline)) goto done;
    int64_t declared = esp_http_client_fetch_headers(client);
    if (declared < 0 || esp_timer_get_time() >= deadline) goto done;
    result = esp_http_client_get_status_code(client);
    if (result != 200) goto done;
    if (declared > (int64_t)capacity) { result = -2; goto done; }
    body_reader_t reader = {.client = client, .deadline = deadline};
    int body_result = p4desk_read_http_body(&reader, read_body, body_complete, out, capacity, length);
    if (body_result) result = body_result;
done: ;
    // Read/drain succeeded completely, including data prefetched with headers.
    // A partial/error response must never become the next request's input.
    bool retained = result == 200 && session.client &&
                    esp_http_client_is_complete_data_received(session.client) &&
                    esp_http_client_is_persistent_connection(session.client);
    if (!retained) p4desk_monitor_http_reset();
    ESP_LOGI("p4desk_monitor", "https end result=%ld bytes=%zu reused=%u retained=%u duration_ms=%lld internal_free=%zu dma_free=%zu stack_free=%u",
             (long)result, *length, (unsigned)reused, (unsigned)retained,
             (long long)((esp_timer_get_time() - began) / 1000),
             heap_caps_get_free_size(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT),
             heap_caps_get_free_size(MALLOC_CAP_INTERNAL | MALLOC_CAP_DMA),
             (unsigned)uxTaskGetStackHighWaterMark(NULL));
    if (result != 200) *length = 0;
    return result;
}
