#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "esp_err.h"

typedef struct p4desk_ppa p4desk_ppa_t;

// One persistent SRM client, used only by the display owner task.
esp_err_t p4desk_ppa_create(p4desk_ppa_t **result);

// Source stays immutable until this call completes. Destination must be a
// FREE/BUILDING LCD framebuffer; never a pending/scanning/retiring buffer.
// Crop the visible 1024x600 block before rotating, ignoring decoded MCU padding.
// On ESP_ERR_TIMEOUT/ESP_ERR_INVALID_STATE, DMA ownership is uncertain: the
// caller must reset/abort rather than fall back, free, or reuse either buffer.
esp_err_t p4desk_ppa_copy_rgb565(p4desk_ppa_t *ppa, uint16_t *destination,
                               size_t destination_bytes, const uint16_t *source,
                               size_t source_bytes, uint32_t source_stride,
                               uint32_t source_rows, bool rotate_180,
                               uint32_t timeout_ms);
