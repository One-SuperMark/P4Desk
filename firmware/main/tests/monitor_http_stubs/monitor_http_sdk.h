#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef int esp_err_t;
#define ESP_OK 0
#define ESP_FAIL -1
#define ESP_LOG_NONE 0
#define MALLOC_CAP_INTERNAL 1
#define MALLOC_CAP_8BIT 2
#define MALLOC_CAP_DMA 4
typedef enum { HTTP_METHOD_GET, HTTP_METHOD_POST } esp_http_client_method_t;
typedef struct fake_client *esp_http_client_handle_t;
typedef struct {
    const char *url;
    esp_http_client_method_t method;
    int timeout_ms;
    bool disable_auto_redirect;
    esp_err_t (*crt_bundle_attach)(void *);
    int buffer_size, buffer_size_tx;
    bool keep_alive_enable;
} esp_http_client_config_t;
esp_http_client_handle_t esp_http_client_init(const esp_http_client_config_t *);
esp_err_t esp_http_client_set_url(esp_http_client_handle_t, const char *);
esp_err_t esp_http_client_set_method(esp_http_client_handle_t, esp_http_client_method_t);
esp_err_t esp_http_client_set_header(esp_http_client_handle_t, const char *, const char *);
esp_err_t esp_http_client_get_header(esp_http_client_handle_t, const char *, char **);
esp_err_t esp_http_client_set_timeout_ms(esp_http_client_handle_t, int);
esp_err_t esp_http_client_open(esp_http_client_handle_t, int);
int esp_http_client_write(esp_http_client_handle_t, const char *, int);
int64_t esp_http_client_fetch_headers(esp_http_client_handle_t);
int esp_http_client_get_status_code(esp_http_client_handle_t);
int esp_http_client_read(esp_http_client_handle_t, char *, int);
bool esp_http_client_is_complete_data_received(esp_http_client_handle_t);
bool esp_http_client_is_persistent_connection(esp_http_client_handle_t);
esp_err_t esp_http_client_cleanup(esp_http_client_handle_t);
esp_err_t esp_crt_bundle_attach(void *);
int64_t esp_timer_get_time(void);
void esp_log_level_set(const char *, int);
size_t heap_caps_get_free_size(unsigned);
size_t heap_caps_get_largest_free_block(unsigned);
unsigned uxTaskGetStackHighWaterMark(void *);
void mock_log(const char *, const char *, ...);
#define ESP_LOGI(tag, ...) mock_log(tag, __VA_ARGS__)
