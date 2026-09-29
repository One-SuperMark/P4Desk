#include "display_ppa.h"
#include "board_p4.h"

#include <stdlib.h>
#include "driver/ppa.h"
#include "esp_attr.h"
#include "esp_cache.h"
#include "esp_heap_caps.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"

struct p4desk_ppa {
    ppa_client_handle_t client;
    SemaphoreHandle_t complete;
    bool poisoned; // Owner task only; remains set after an uncertain DMA timeout.
};

static bool IRAM_ATTR transaction_done(ppa_client_handle_t client,
                                     ppa_event_data_t *event, void *context)
{
    (void)client; (void)event;
    p4desk_ppa_t *ppa = context;
    BaseType_t wake = pdFALSE;
    xSemaphoreGiveFromISR(ppa->complete, &wake);
    return wake == pdTRUE;
}

esp_err_t p4desk_ppa_create(p4desk_ppa_t **result)
{
    if (!result) return ESP_ERR_INVALID_ARG;
    if (*result) return ESP_ERR_INVALID_STATE;
    // Callback context and semaphore must remain available to the DMA ISR.
    p4desk_ppa_t *ppa = heap_caps_calloc(1, sizeof(*ppa), MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT);
    if (!ppa) return ESP_ERR_NO_MEM;
    ppa->complete = xSemaphoreCreateBinary();
    if (!ppa->complete) { free(ppa); return ESP_ERR_NO_MEM; }

    const ppa_client_config_t config = {
        .oper_type = PPA_OPERATION_SRM,
        .max_pending_trans_num = 1,
        .data_burst_length = PPA_DATA_BURST_LENGTH_128,
    };
    esp_err_t ret = ppa_register_client(&config, &ppa->client);
    if (ret != ESP_OK) {
        vSemaphoreDelete(ppa->complete);
        free(ppa);
        return ret;
    }
    const ppa_event_callbacks_t callbacks = {.on_trans_done = transaction_done};
    ret = ppa_client_register_event_callbacks(ppa->client, &callbacks);
    if (ret != ESP_OK) {
        ppa_unregister_client(ppa->client);
        vSemaphoreDelete(ppa->complete);
        free(ppa);
        return ret;
    }
    *result = ppa;
    return ESP_OK;
}

esp_err_t p4desk_ppa_copy_rgb565(p4desk_ppa_t *ppa, uint16_t *destination,
                               size_t destination_bytes, const uint16_t *source,
                               size_t source_bytes, uint32_t source_stride,
                               uint32_t source_rows, bool rotate_180,
                               uint32_t timeout_ms)
{
    if (!ppa || !destination || !source || !timeout_ms) return ESP_ERR_INVALID_ARG;
    if (ppa->poisoned) return ESP_ERR_INVALID_STATE;
    const uint64_t source_span = (uint64_t)source_stride * source_rows * sizeof(uint16_t);
    if (source_stride < P4DESK_WIDTH || source_rows < P4DESK_HEIGHT ||
        source_span > source_bytes || destination_bytes < P4DESK_FB_BYTES ||
        destination_bytes > UINT32_MAX) return ESP_ERR_INVALID_ARG;

    // PPA invalidates its output before DMA. Source/destination overlap would
    // destroy source pixels or race the transaction, so prohibit in-place SRM.
    const uintptr_t input = (uintptr_t)source, output = (uintptr_t)destination;
    if ((output >= input && output - input < source_span) ||
        (input > output && input - output < destination_bytes)) return ESP_ERR_INVALID_ARG;
    const size_t alignment = esp_cache_get_line_size_by_addr(destination);
    if (!alignment || output % alignment || destination_bytes % alignment) return ESP_ERR_INVALID_ARG;

    const ppa_srm_oper_config_t config = {
        .in = {
            .buffer = source, .pic_w = source_stride, .pic_h = source_rows,
            .block_w = P4DESK_WIDTH, .block_h = P4DESK_HEIGHT,
            .block_offset_x = 0, .block_offset_y = 0,
            .srm_cm = PPA_SRM_COLOR_MODE_RGB565,
        },
        .out = {
            .buffer = destination, .buffer_size = destination_bytes,
            .pic_w = P4DESK_WIDTH, .pic_h = P4DESK_HEIGHT,
            .block_offset_x = 0, .block_offset_y = 0,
            .srm_cm = PPA_SRM_COLOR_MODE_RGB565,
        },
        .rotation_angle = rotate_180 ? PPA_SRM_ROTATION_ANGLE_180 : PPA_SRM_ROTATION_ANGLE_0,
        .scale_x = 1.0f, .scale_y = 1.0f,
        .mirror_x = false, .mirror_y = false,
        .rgb_swap = false, .byte_swap = false,
        .alpha_update_mode = PPA_ALPHA_NO_CHANGE,
        // Nonblocking driver submission lets this owner enforce a deadline.
        .mode = PPA_TRANS_MODE_NON_BLOCKING, .user_data = ppa,
    };
    // The IDF driver performs input C2M and output M2C cache synchronization.
    // No CPU may read/write the output between submission and completion.
    xSemaphoreTake(ppa->complete, 0);
    esp_err_t ret = ppa_do_scale_rotate_mirror(ppa->client, &config);
    if (ret != ESP_OK) return ret; // Rejected before DMA submission: CPU fallback is safe.
    if (xSemaphoreTake(ppa->complete, pdMS_TO_TICKS(timeout_ms)) != pdTRUE) {
        ppa->poisoned = true;
        return ESP_ERR_TIMEOUT;
    }
    return ESP_OK;
}
