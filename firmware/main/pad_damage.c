#include "pad_damage.h"

#include <limits.h>
#include <string.h>

static bool valid_dimensions(const p4pad_damage_t *damage)
{
    return damage && damage->width > 0 && damage->height > 0 && damage->band_height > 0 &&
        damage->band_height == (int32_t)(((uint32_t)damage->height + P4PAD_DAMAGE_BANDS - 1) / P4PAD_DAMAGE_BANDS) &&
        (size_t)damage->width <= SIZE_MAX / sizeof(uint16_t) / (size_t)damage->height;
}

static bool valid_rect(const p4pad_damage_t *damage, const p4pad_rect_t *rect)
{
    return valid_dimensions(damage) && rect && rect->x1 >= 0 && rect->y1 >= 0 &&
        rect->x1 < rect->x2 && rect->y1 < rect->y2 &&
        rect->x2 <= damage->width && rect->y2 <= damage->height;
}

bool p4pad_damage_init(p4pad_damage_t *damage, uint32_t width, uint32_t height)
{
    if (!damage) return false;
    memset(damage, 0, sizeof(*damage));
    if (!width || !height || width > INT32_MAX || height > INT32_MAX ||
        (size_t)width > SIZE_MAX / sizeof(uint16_t) / (size_t)height) return false;
    damage->width = (int32_t)width;
    damage->height = (int32_t)height;
    damage->band_height = (int32_t)(((uint32_t)height + P4PAD_DAMAGE_BANDS - 1) / P4PAD_DAMAGE_BANDS);
    p4pad_damage_invalidate_all(damage);
    return true;
}

void p4pad_damage_invalidate_all(p4pad_damage_t *damage)
{
    if (!valid_dimensions(damage)) return;
    const p4pad_rect_t full = {0, 0, damage->width, damage->height};
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        damage->pending[i] = full;
        memset(damage->bands[i], 0, sizeof(damage->bands[i]));
        for (unsigned band = 0; band < P4PAD_DAMAGE_BANDS; ++band) {
            const int64_t y1 = (int64_t)band * damage->band_height;
            if (y1 >= damage->height) break;
            const int64_t y2 = y1 + damage->band_height;
            damage->bands[i][band] = (p4pad_rect_t){0, (int32_t)y1, damage->width,
                (int32_t)(y2 < damage->height ? y2 : damage->height)};
        }
    }
}

static void union_rect(const p4pad_damage_t *damage, p4pad_rect_t *pending,
                       const p4pad_rect_t *rect)
{
    if (!valid_rect(damage, pending)) { *pending = *rect; return; }
    if (rect->x1 < pending->x1) pending->x1 = rect->x1;
    if (rect->y1 < pending->y1) pending->y1 = rect->y1;
    if (rect->x2 > pending->x2) pending->x2 = rect->x2;
    if (rect->y2 > pending->y2) pending->y2 = rect->y2;
}

bool p4pad_damage_mark(p4pad_damage_t *damage,
                      int32_t x1, int32_t y1, int32_t x2, int32_t y2)
{
    if (!valid_dimensions(damage) || x1 >= x2 || y1 >= y2) return false;
    if (x1 < 0) x1 = 0;
    if (y1 < 0) y1 = 0;
    if (x2 > damage->width) x2 = damage->width;
    if (y2 > damage->height) y2 = damage->height;
    const p4pad_rect_t rect = {x1, y1, x2, y2};
    if (!valid_rect(damage, &rect)) return false;
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        union_rect(damage, &damage->pending[i], &rect);
        const unsigned first = (unsigned)(y1 / damage->band_height);
        const unsigned last = (unsigned)((y2 - 1) / damage->band_height);
        for (unsigned band = first; band <= last; ++band) {
            const int64_t top = (int64_t)band * damage->band_height;
            const int64_t bottom = top + damage->band_height;
            const p4pad_rect_t part = {x1, top > y1 ? (int32_t)top : y1, x2,
                bottom < y2 ? (int32_t)bottom : y2};
            union_rect(damage, &damage->bands[i][band], &part);
        }
    }
    return true;
}

bool p4pad_damage_peek(const p4pad_damage_t *damage, unsigned index, p4pad_rect_t *rect)
{
    if (!rect) return false;
    *rect = (p4pad_rect_t){0};
    if (!valid_dimensions(damage) || index >= P4PAD_DAMAGE_BUFFER_COUNT ||
        !valid_rect(damage, &damage->pending[index])) return false;
    *rect = damage->pending[index];
    return true;
}

bool p4pad_damage_take(p4pad_damage_t *damage, unsigned index, p4pad_rect_t *rect)
{
    if (!p4pad_damage_peek(damage, index, rect)) return false;
    damage->pending[index] = (p4pad_rect_t){0};
    memset(damage->bands[index], 0, sizeof(damage->bands[index]));
    return true;
}

bool p4pad_damage_take_batch(p4pad_damage_t *damage, unsigned index,
                           p4pad_damage_batch_t *batch)
{
    if (!batch) return false;
    *batch = (p4pad_damage_batch_t){0};
    if (!p4pad_damage_peek(damage, index, &batch->bounds)) return false;
    for (unsigned band = 0; band < P4PAD_DAMAGE_BANDS; ++band) {
        const p4pad_rect_t rect = damage->bands[index][band];
        if (!valid_rect(damage, &rect)) continue;
        batch->pixels += (size_t)(rect.x2 - rect.x1) * (size_t)(rect.y2 - rect.y1);
        p4pad_rect_t *last = batch->count ? &batch->regions[batch->count - 1] : NULL;
        if (last && last->x1 == rect.x1 && last->x2 == rect.x2 && last->y2 == rect.y1) {
            last->y2 = rect.y2;
        } else {
            batch->regions[batch->count++] = rect;
        }
    }
    // State is produced exclusively by init/mark; never discard valid debt if
    // that invariant is violated. The caller treats this as no prepared frame.
    if (!batch->count) return false;
    damage->pending[index] = (p4pad_rect_t){0};
    memset(damage->bands[index], 0, sizeof(damage->bands[index]));
    return true;
}

bool p4pad_damage_map_rect(const p4pad_damage_t *damage, const p4pad_rect_t *source,
                          bool rotate_180, p4pad_rect_t *destination)
{
    if (!destination || !valid_rect(damage, source)) return false;
    // Read first so callers may pass the same rectangle for input and output.
    const p4pad_rect_t rect = *source;
    *destination = rotate_180 ? (p4pad_rect_t){
        damage->width - rect.x2, damage->height - rect.y2,
        damage->width - rect.x1, damage->height - rect.y1,
    } : rect;
    return true;
}

static bool required_pixels(const p4pad_damage_t *damage, size_t stride, size_t *required)
{
    if (!valid_dimensions(damage) || stride < (size_t)damage->width) return false;
    const size_t rows = (size_t)damage->height - 1;
    const size_t width = (size_t)damage->width;
    if (rows && stride > (SIZE_MAX - width) / rows) return false;
    *required = rows * stride + width;
    return *required <= SIZE_MAX / sizeof(uint16_t);
}

bool p4pad_damage_row_span(const p4pad_damage_t *damage, const p4pad_rect_t *source,
                          bool rotate_180, size_t destination_stride,
                          size_t *first, size_t *past_last)
{
    size_t required;
    p4pad_rect_t mapped;
    if (!first || !past_last || !required_pixels(damage, destination_stride, &required) ||
        !p4pad_damage_map_rect(damage, source, rotate_180, &mapped)) return false;
    *first = (size_t)mapped.y1 * destination_stride + (size_t)mapped.x1;
    *past_last = ((size_t)mapped.y2 - 1) * destination_stride + (size_t)mapped.x2;
    return *past_last <= required;
}

bool p4pad_damage_copy_rgb565(const p4pad_damage_t *damage,
                            uint16_t *destination, size_t destination_count,
                            size_t destination_stride,
                            const uint16_t *source, size_t source_count,
                            size_t source_stride, const p4pad_rect_t *rect,
                            bool rotate_180)
{
    size_t source_required, destination_required;
    if (!destination || !source || !valid_rect(damage, rect) ||
        !required_pixels(damage, source_stride, &source_required) ||
        !required_pixels(damage, destination_stride, &destination_required) ||
        source_required > source_count || destination_required > destination_count) return false;
    const uintptr_t source_start = (uintptr_t)source;
    const uintptr_t destination_start = (uintptr_t)destination;
    const size_t source_bytes = source_required * sizeof(uint16_t);
    const size_t destination_bytes = destination_required * sizeof(uint16_t);
    if (source_start > UINTPTR_MAX - source_bytes ||
        destination_start > UINTPTR_MAX - destination_bytes) return false;
    if (source_start < destination_start + destination_bytes &&
        destination_start < source_start + source_bytes) return false;
    const size_t width = (size_t)(rect->x2 - rect->x1);
    for (int32_t y = rect->y1; y < rect->y2; ++y) {
        const uint16_t *input = source + (size_t)y * source_stride + (size_t)rect->x1;
        const int32_t destination_y = rotate_180 ? damage->height - 1 - y : y;
        uint16_t *output = destination + (size_t)destination_y * destination_stride;
        if (!rotate_180) {
            memcpy(output + rect->x1, input, width * sizeof(uint16_t));
        } else {
            for (int32_t x = rect->x1; x < rect->x2; ++x)
                output[damage->width - 1 - x] = input[x - rect->x1];
        }
    }
    return true;
}
