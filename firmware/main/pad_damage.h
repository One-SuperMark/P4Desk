#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// All rectangles use canonical Pad/source coordinates and exclusive ends.
// The caller owns synchronization: mark and take must run under the same source
// mutex as the pixel writes/copy. No heap allocation or framebuffer is stored.
#define P4PAD_DAMAGE_BUFFER_COUNT 3
#define P4PAD_DAMAGE_BANDS 32

typedef struct {
    int32_t x1, y1, x2, y2;
} p4pad_rect_t;

typedef struct {
    int32_t width, height;
    int32_t band_height;
    p4pad_rect_t pending[P4PAD_DAMAGE_BUFFER_COUNT];
    p4pad_rect_t bands[P4PAD_DAMAGE_BUFFER_COUNT][P4PAD_DAMAGE_BANDS];
} p4pad_damage_t;

typedef struct {
    p4pad_rect_t bounds;
    p4pad_rect_t regions[P4PAD_DAMAGE_BANDS];
    size_t count, pixels;
} p4pad_damage_batch_t;

// Each framebuffer starts with a full debt; invalid initialization clears state.
bool p4pad_damage_init(p4pad_damage_t *damage, uint32_t width, uint32_t height);
void p4pad_damage_invalidate_all(p4pad_damage_t *damage);

// Clip to the screen, then union into each framebuffer's independent debt.
// Empty or entirely offscreen rectangles return false without changing debt.
bool p4pad_damage_mark(p4pad_damage_t *damage,
                      int32_t x1, int32_t y1, int32_t x2, int32_t y2);
bool p4pad_damage_peek(const p4pad_damage_t *damage, unsigned index, p4pad_rect_t *rect);
bool p4pad_damage_take(p4pad_damage_t *damage, unsigned index, p4pad_rect_t *rect);
// Non-overlapping horizontal bands retain distant damage without copying the
// unchanged space between it. Adjacent equal-width regions coalesce. No heap.
bool p4pad_damage_take_batch(p4pad_damage_t *damage, unsigned index,
                           p4pad_damage_batch_t *batch);

// Map canonical damage to its destination rectangle. Rotation is 0 or 180 only.
bool p4pad_damage_map_rect(const p4pad_damage_t *damage, const p4pad_rect_t *source,
                          bool rotate_180, p4pad_rect_t *destination);

// Contiguous destination pixel span covering all touched rows, including gaps
// between row segments. Both outputs are indices, with past_last exclusive.
bool p4pad_damage_row_span(const p4pad_damage_t *damage, const p4pad_rect_t *source,
                          bool rotate_180, size_t destination_stride,
                          size_t *first, size_t *past_last);

// Copy just this debt into a separately owned destination buffer. Counts and
// strides are in pixels, not bytes. Row/MCU padding remains unchanged; invalid
// dimensions/capacity, overflow, and overlapping buffers are rejected before any
// write. This helper does not flush cache or change framebuffer ownership.
bool p4pad_damage_copy_rgb565(const p4pad_damage_t *damage,
                            uint16_t *destination, size_t destination_count,
                            size_t destination_stride,
                            const uint16_t *source, size_t source_count,
                            size_t source_stride, const p4pad_rect_t *rect,
                            bool rotate_180);
