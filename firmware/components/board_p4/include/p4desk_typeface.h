/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
    P4DESK_TYPEFACE_OK = 0,
    P4DESK_TYPEFACE_MISSING = 1,
    P4DESK_TYPEFACE_UNAVAILABLE = 2,
    P4DESK_TYPEFACE_INVALID = 3,
    P4DESK_TYPEFACE_BUFFER_SMALL = 4,
    P4DESK_TYPEFACE_RENDER_FAILED = 5,
    P4DESK_TYPEFACE_BUDGET = 6,
};

/* Coordinates use the same bottom bearing as tiny-flutter/fontdue. */
typedef struct {
    uint16_t width;
    uint16_t height;
    int16_t xmin;
    int16_t ymin;
    float advance;
} p4desk_typeface_metrics_t;

typedef struct {
    uint32_t current_bytes;
    uint32_t peak_bytes;
    uint32_t allocation_failures;
    uint32_t successful_allocations;
} p4desk_typeface_memory_t;

/* Call from the resource worker after TF mounting. It may be retried after
 * failure. Only three fixed paths are accepted; caller-supplied paths are absent.
 * The font remains file-backed, with one open TF descriptor. */
int32_t p4desk_typeface_init(void);
bool p4desk_typeface_ready(void);

/* These never initialize/open a font on the UI thread. Output is zeroed on
 * failure. Metrics and render share one locked FreeType face. */
int32_t p4desk_typeface_metrics(uint32_t codepoint, uint16_t px,
                               p4desk_typeface_metrics_t *out);
int32_t p4desk_typeface_render(uint32_t codepoint, uint16_t px,
                              uint8_t *alpha, size_t capacity,
                              p4desk_typeface_metrics_t *out);
void p4desk_typeface_memory(p4desk_typeface_memory_t *out);

#ifdef __cplusplus
}
#endif
