// Exercise the production transport against a bounded SDK double. This tests
// ownership/isolation/error policy; the pinned SDK/device validate actual TLS.
#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "monitor_http_sdk.h"
#include "../p4desk_monitor_http.c"

typedef struct {
    const char *data;
    size_t length, fragment;
    int status;
    int64_t declared, open_us, fetch_us, read_us;
    bool persistent, complete, open_error, write_error, read_error;
} response_t;
struct fake_client {
    char url[4097], key[513], written[128];
    esp_http_client_method_t method;
    int timeout_ms, body_length, written_length;
    size_t position;
    response_t response;
};
static response_t responses[32];
static size_t response_count, next_response;
static unsigned initialized, cleaned, opened, active;
static int64_t now_us;
static bool init_error, header_error, url_error, method_error;
static struct fake_client *last_client;

static response_t good(const char *data)
{
    return (response_t){.data = data, .length = strlen(data), .fragment = 2,
        .status = 200, .declared = (int64_t)strlen(data), .persistent = true, .complete = true};
}
static void setup(void)
{
    p4desk_monitor_http_reset();
    assert(active == 0);
    memset(responses, 0, sizeof(responses));
    response_count = next_response = 0;
    initialized = cleaned = opened = 0;
    now_us = 1000;
    init_error = header_error = url_error = method_error = false;
    last_client = NULL;
}
static void enqueue(response_t response)
{
    assert(response_count < 32);
    responses[response_count++] = response;
}
static int request(const char *url, const char *key, const char *body,
                   uint8_t *out, size_t capacity, size_t *length)
{
    return p4desk_monitor_http(url, key, body, out, capacity, length);
}
esp_http_client_handle_t esp_http_client_init(const esp_http_client_config_t *c)
{
    assert(c->disable_auto_redirect && c->keep_alive_enable);
    assert(c->crt_bundle_attach == esp_crt_bundle_attach);
    assert(c->buffer_size == 2048 && c->buffer_size_tx == 2048 && c->timeout_ms == 6000);
    initialized++;
    if (init_error) return NULL;
    struct fake_client *client = calloc(1, sizeof(*client));
    assert(client);
    strcpy(client->url, c->url);
    client->method = c->method;
    active++;
    last_client = client;
    return client;
}
esp_err_t esp_http_client_set_url(esp_http_client_handle_t client, const char *url)
{
    if (url_error) return ESP_FAIL;
    assert(strlen(url) < sizeof(client->url));
    strcpy(client->url, url);
    return ESP_OK;
}
esp_err_t esp_http_client_set_method(esp_http_client_handle_t client, esp_http_client_method_t method)
{
    if (method_error) return ESP_FAIL;
    client->method = method;
    return ESP_OK;
}
esp_err_t esp_http_client_set_header(esp_http_client_handle_t client, const char *key, const char *value)
{
    if (header_error) return ESP_FAIL;
    if (strcmp(key, "x-api-key") == 0) strcpy(client->key, value);
    return ESP_OK;
}
esp_err_t esp_http_client_get_header(esp_http_client_handle_t client, const char *key, char **value)
{
    *value = strcmp(key, "x-api-key") == 0 && client->key[0] ? client->key : NULL;
    return ESP_OK;
}
esp_err_t esp_http_client_set_timeout_ms(esp_http_client_handle_t client, int timeout)
{
    assert(timeout > 0 && timeout <= 6000);
    client->timeout_ms = timeout;
    return ESP_OK;
}
esp_err_t esp_http_client_open(esp_http_client_handle_t client, int write_len)
{
    assert(next_response < response_count);
    client->response = responses[next_response++];
    client->position = 0;
    client->body_length = write_len;
    client->written_length = 0;
    memset(client->written, 0, sizeof(client->written));
    opened++;
    now_us += client->response.open_us;
    return client->response.open_error ? ESP_FAIL : ESP_OK;
}
int esp_http_client_write(esp_http_client_handle_t client, const char *data, int length)
{
    if (client->response.write_error) return -1;
    int n = length > 3 ? 3 : length;
    assert(client->written_length + n < (int)sizeof(client->written));
    memcpy(client->written + client->written_length, data, (size_t)n);
    client->written_length += n;
    return n;
}
int64_t esp_http_client_fetch_headers(esp_http_client_handle_t client)
{
    now_us += client->response.fetch_us;
    return client->response.declared;
}
int esp_http_client_get_status_code(esp_http_client_handle_t client) { return client->response.status; }
int esp_http_client_read(esp_http_client_handle_t client, char *out, int capacity)
{
    now_us += client->response.read_us;
    if (client->response.read_error) return -1;
    size_t count = client->response.length - client->position;
    if (count > client->response.fragment) count = client->response.fragment;
    if (count > (size_t)capacity) count = (size_t)capacity;
    memcpy(out, client->response.data + client->position, count);
    client->position += count;
    return (int)count;
}
bool esp_http_client_is_complete_data_received(esp_http_client_handle_t client)
{
    // The real SDK parser can report complete before cached bytes are drained.
    return client->response.complete;
}
bool esp_http_client_is_persistent_connection(esp_http_client_handle_t client)
{
    return client->response.persistent;
}
esp_err_t esp_http_client_cleanup(esp_http_client_handle_t client)
{
    for (size_t i = 0; i < sizeof(client->key); i++) assert(client->key[i] == 0);
    free(client);
    active--;
    cleaned++;
    return ESP_OK;
}
esp_err_t esp_crt_bundle_attach(void *unused) { (void)unused; return ESP_OK; }
int64_t esp_timer_get_time(void) { return now_us; }
void esp_log_level_set(const char *tag, int level) { assert(strcmp(tag, "HTTP_CLIENT") == 0 && level == ESP_LOG_NONE); }
size_t heap_caps_get_free_size(unsigned caps) { (void)caps; return 70000; }
size_t heap_caps_get_largest_free_block(unsigned caps) { (void)caps; return 65536; }
unsigned uxTaskGetStackHighWaterMark(void *task) { (void)task; return 8000; }
void mock_log(const char *tag, const char *format, ...) { (void)tag; (void)format; }

static void test_reuse_and_methods(void)
{
    setup();
    uint8_t out[16]; size_t length;
    enqueue(good("abcdef")); enqueue(good("second")); enqueue(good("third"));
    assert(request("https://a.example/api/stats?day=1", "test-key-a", NULL, out, 6, &length) == 200);
    assert(length == 6 && memcmp(out, "abcdef", 6) == 0 && active == 1 && initialized == 1);
    assert(request("https://a.example/api/batch", "test-key-a", "{\"ids\":[]}", out, 16, &length) == 200);
    assert(initialized == 1 && opened == 2 && cleaned == 0);
    assert(last_client->method == HTTP_METHOD_POST && last_client->body_length == 10);
    assert(strcmp(last_client->written, "{\"ids\":[]}") == 0);
    assert(strcmp(last_client->url, "https://a.example/api/batch") == 0);
    assert(request("https://a.example/api/stats", "test-key-a", NULL, out, 16, &length) == 200);
    assert(last_client->method == HTTP_METHOD_GET && last_client->body_length == 0 && last_client->written_length == 0);
    assert(initialized == 1 && opened == 3);
    p4desk_monitor_http_reset();
    assert(active == 0 && cleaned == 1 && session.key[0] == 0);
    p4desk_monitor_http_reset();
    assert(cleaned == 1);
}
static void test_identity(void)
{
    setup();
    uint8_t out[16]; size_t length;
    for (int i = 0; i < 5; i++) enqueue(good("ok"));
    assert(request("https://a.example/api/stats", "test-key-a", NULL, out, 16, &length) == 200);
    assert(request("https://a.example/api/stats", "test-key-b", NULL, out, 16, &length) == 200);
    assert(initialized == 2 && cleaned == 1 && strcmp(session.key, "test-key-b") == 0);
    assert(request("https://b.example/api/stats", "test-key-b", NULL, out, 16, &length) == 200);
    assert(initialized == 3 && cleaned == 2);
    assert(request("https://b.example:8443/api/stats", "test-key-b", NULL, out, 16, &length) == 200);
    assert(initialized == 4 && cleaned == 3);
    assert(request("https://b.example:443/api/stats", "test-key-b", NULL, out, 16, &length) == 200);
    assert(initialized == 5 && cleaned == 4);
    p4desk_monitor_http_reset();
}
static void test_failure_cleanup(void)
{
    uint8_t out[16]; size_t length;
    for (int scenario = 0; scenario < 9; scenario++) {
        setup();
        enqueue(good("ok"));
        assert(request("https://a.example/api", "test-key", NULL, out, 16, &length) == 200);
        response_t failed = good("abcde");
        int result = -1;
        switch (scenario) {
            case 0: failed.status = 401; result = 401; break;
            case 1: failed.status = 302; result = 302; break;
            case 2: failed.declared = 17; result = -2; break;
            case 3: failed.declared = 0; failed.length = 5; result = -2; break;
            case 4: failed.complete = false; result = -3; break;
            case 5: failed.read_error = true; break;
            case 6: failed.open_error = true; break;
            case 7: failed.write_error = true; break;
            case 8: failed.declared = -1; break;
        }
        enqueue(failed);
        assert(request("https://a.example/api", "test-key", scenario == 7 ? "{}" : NULL,
                       out, scenario == 3 ? 4 : 16, &length) == result);
        assert(length == 0 && active == 0 && cleaned == 1 && opened == 2);
        // No automatic replay; a later independent call creates a fresh client.
        enqueue(good("recovered"));
        assert(request("https://a.example/api", "test-key", NULL, out, 16, &length) == 200);
        assert(initialized == 2 && opened == 3);
        p4desk_monitor_http_reset();
    }
}
static void test_server_close_and_deadline(void)
{
    setup();
    uint8_t out[16]; size_t length;
    response_t close = good("ok"); close.persistent = false;
    enqueue(close); enqueue(good("ok"));
    assert(request("https://a.example/api", "test-key", NULL, out, 16, &length) == 200 && length == 2);
    assert(active == 0 && cleaned == 1);
    assert(request("https://a.example/api", "test-key", NULL, out, 16, &length) == 200);
    assert(initialized == 2);
    p4desk_monitor_http_reset();
    for (int phase = 0; phase < 3; phase++) {
        setup();
        response_t delayed = good("ok");
        if (phase == 0) delayed.open_us = 12000000;
        if (phase == 1) delayed.fetch_us = 12000000;
        if (phase == 2) delayed.read_us = 12000000;
        enqueue(delayed);
        assert(request("https://a.example/api", "test-key", NULL, out, 16, &length) == -1);
        assert(length == 0 && active == 0 && cleaned == 1 && opened == 1);
    }
}
static void test_validation_and_sdk_setup_errors(void)
{
    uint8_t out[16]; size_t length;
    const char *invalid_urls[] = {"http://a.example/api", "https:///api", "https://user@a.example/api",
        "https://a.example/api#fragment", "https://a.example/a b", "https://a.example/\\bad"};
    for (size_t i = 0; i < sizeof(invalid_urls) / sizeof(invalid_urls[0]); i++) {
        setup();
        assert(request(invalid_urls[i], "test-key", NULL, out, 16, &length) == -1 && length == 0);
        assert(initialized == 0);
    }
    setup();
    char long_key[514]; memset(long_key, 'a', sizeof(long_key) - 1); long_key[513] = 0;
    assert(request("https://a.example/api", long_key, NULL, out, 16, &length) == -1);
    assert(request("https://a.example/api", "bad\r\nkey", NULL, out, 16, &length) == -1);
    char *body = malloc(16386); assert(body); memset(body, 'x', 16385); body[16385] = 0;
    assert(request("https://a.example/api", "test-key", body, out, 16, &length) == -1);
    free(body);
    assert(initialized == 0);
    for (int error = 0; error < 4; error++) {
        setup();
        init_error = error == 0; header_error = error == 1;
        url_error = error == 2; method_error = error == 3;
        assert(request("https://a.example/api", "test-key", NULL, out, 16, &length) == -1);
        assert(length == 0 && active == 0 && opened == 0);
    }
}
static void test_existing_config_boundaries(void)
{
    setup();
    uint8_t out[16]; size_t length;
    // Config::new permits 240 bytes for the entire site and a 512-byte key.
    char url[4098], key[513];
    memcpy(url, "https://", 8);
    memset(url + 8, 'a', 232);
    memcpy(url + 240, "/api/v1/admin/dashboard/stats", 30);
    memset(key, 'k', 512); key[512] = 0;
    enqueue(good("ok"));
    assert(request(url, key, NULL, out, sizeof(out), &length) == 200);
    assert(strlen(session.authority) == 240 && strlen(session.key) == 512);
    p4desk_monitor_http_reset();
    // The path/query has its own limit and does not shrink the site's allowance.
    memcpy(url, "https://a.example/", 18);
    memset(url + 18, 'p', 4096 - 18); url[4096] = 0;
    size_t authority_length;
    assert(request_identity(url, key, &authority_length));
    url[4096] = 'p'; url[4097] = 0;
    assert(!request_identity(url, key, &authority_length));
}
int main(void)
{
    test_reuse_and_methods();
    test_identity();
    test_failure_cleanup();
    test_server_close_and_deadline();
    test_validation_and_sdk_setup_errors();
    test_existing_config_boundaries();
    p4desk_monitor_http_reset();
    assert(active == 0);
    puts("HTTPS session: reuse, method/path changes, origin/key isolation, exact/overflow bounds, failure cleanup, close, deadlines and validation passed");
}
