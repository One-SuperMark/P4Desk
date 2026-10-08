/* SPDX-License-Identifier: Apache-2.0 */
#include "p4desk_typeface.h"

#include <limits.h>
#include <math.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "esp_heap_caps.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "ft2build.h"
#include FT_FREETYPE_H
#include FT_MODULE_H
#include FT_OUTLINE_H

/* Includes allocation headers and realloc's transient old/new overlap. */
#define TYPEFACE_MEMORY_LIMIT (2u * 1024u * 1024u)
#define TYPEFACE_FILE_LIMIT (32u * 1024u * 1024u)
#define TYPEFACE_LOCK_MS 20u

_Static_assert(sizeof(p4desk_typeface_metrics_t) == 12, "typeface FFI metrics size");
_Static_assert(offsetof(p4desk_typeface_metrics_t, advance) == 8, "typeface FFI advance offset");
_Static_assert(sizeof(p4desk_typeface_memory_t) == 16, "typeface FFI memory size");

typedef union {
    struct { size_t total; } value;
    max_align_t alignment;
} allocation_header_t;

static struct FT_MemoryRec_ s_ft_memory;
static FT_Library s_library;
static FT_Face s_face;
static p4desk_typeface_memory_t s_memory;
static StaticSemaphore_t s_mutex_storage;
static SemaphoreHandle_t s_mutex;
static portMUX_TYPE s_mutex_init_lock = portMUX_INITIALIZER_UNLOCKED;
static bool s_glyph_valid;
static bool s_glyph_rendered;
static uint32_t s_glyph_codepoint;
static uint16_t s_glyph_px;
static p4desk_typeface_metrics_t s_glyph_metrics;

static SemaphoreHandle_t face_mutex(void)
{
    portENTER_CRITICAL(&s_mutex_init_lock);
    if (s_mutex == NULL) {
        s_mutex = xSemaphoreCreateMutexStatic(&s_mutex_storage);
    }
    SemaphoreHandle_t result = s_mutex;
    portEXIT_CRITICAL(&s_mutex_init_lock);
    return result;
}

static bool lock_face(void)
{
    SemaphoreHandle_t mutex = face_mutex();
    return mutex != NULL && xSemaphoreTake(mutex, pdMS_TO_TICKS(TYPEFACE_LOCK_MS)) == pdTRUE;
}

static void failure_count(void)
{
    if (s_memory.allocation_failures != UINT32_MAX) {
        ++s_memory.allocation_failures;
    }
}

static void *ft_allocate(FT_Memory memory, long size)
{
    (void)memory;
    if (size <= 0 || (unsigned long)size > SIZE_MAX - sizeof(allocation_header_t)) {
        failure_count();
        return NULL;
    }
    const size_t total = (size_t)size + sizeof(allocation_header_t);
    if (total > TYPEFACE_MEMORY_LIMIT || s_memory.current_bytes > TYPEFACE_MEMORY_LIMIT - total) {
        failure_count();
        return NULL;
    }
    allocation_header_t *allocation = heap_caps_malloc(total, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
    if (allocation == NULL) {
        failure_count();
        return NULL;
    }
    allocation->value.total = total;
    s_memory.current_bytes += (uint32_t)total;
    if (s_memory.current_bytes > s_memory.peak_bytes) {
        s_memory.peak_bytes = s_memory.current_bytes;
    }
    if (s_memory.successful_allocations != UINT32_MAX) {
        ++s_memory.successful_allocations;
    }
    return allocation + 1;
}

static void ft_release(FT_Memory memory, void *block)
{
    (void)memory;
    if (block == NULL) {
        return;
    }
    allocation_header_t *allocation = (allocation_header_t *)block - 1;
    s_memory.current_bytes -= (uint32_t)allocation->value.total;
    heap_caps_free(allocation);
}

static void *ft_reallocate(FT_Memory memory, long current_size, long new_size, void *block)
{
    (void)current_size;
    if (new_size <= 0) {
        ft_release(memory, block);
        return NULL;
    }
    if (block == NULL) {
        return ft_allocate(memory, new_size);
    }
    allocation_header_t *old = (allocation_header_t *)block - 1;
    const size_t old_size = old->value.total - sizeof(allocation_header_t);
    if ((unsigned long)new_size <= old_size) {
        /* Retaining the block on shrink avoids both allocation and transient
         * overlap. Its full reserved size remains charged to the budget. */
        return block;
    }
    void *replacement = ft_allocate(memory, new_size);
    if (replacement == NULL) {
        return NULL; /* FreeType retains the original allocation on failure. */
    }
    memcpy(replacement, block, old_size);
    ft_release(memory, block);
    return replacement;
}

static void close_font_locked(void)
{
    s_glyph_valid = false;
    s_glyph_rendered = false;
    if (s_face != NULL) {
        FT_Done_Face(s_face);
        s_face = NULL;
    }
    if (s_library != NULL) {
        FT_Done_Library(s_library);
        s_library = NULL;
    }
}

int32_t p4desk_typeface_init(void)
{
    if (!lock_face()) {
        return P4DESK_TYPEFACE_UNAVAILABLE;
    }
    int32_t result = P4DESK_TYPEFACE_OK;
    if (s_face == NULL) {
        static const char *const paths[] = {
            "/sdcard/fonts/HarmonyOS_Sans_SC_Regular.ttf",
            "/sdcard/fonts/NotoSansCJKsc-Regular.otf",
            "/sdcard/typeface/HarmonyOS_Sans_SC_Regular.ttf",
        };
        s_ft_memory.alloc = ft_allocate;
        s_ft_memory.free = ft_release;
        s_ft_memory.realloc = ft_reallocate;
        const uint32_t before_failures = s_memory.allocation_failures;
        if (FT_New_Library(&s_ft_memory, &s_library) == 0) {
            FT_Add_Default_Modules(s_library);
            for (size_t i = 0; i < sizeof(paths) / sizeof(paths[0]); ++i) {
                struct stat status;
                if (stat(paths[i], &status) != 0 || !S_ISREG(status.st_mode)
                    || status.st_size <= 0 || (uint64_t)status.st_size > TYPEFACE_FILE_LIMIT) {
                    continue;
                }
                FT_Face candidate = NULL;
                if (FT_New_Face(s_library, paths[i], 0, &candidate) != 0) {
                    continue;
                }
                if (!FT_IS_SCALABLE(candidate) || FT_Select_Charmap(candidate, FT_ENCODING_UNICODE) != 0) {
                    FT_Done_Face(candidate);
                    continue;
                }
                s_face = candidate;
                break;
            }
        }
        if (s_face == NULL) {
            close_font_locked();
            result = s_memory.allocation_failures != before_failures
                ? P4DESK_TYPEFACE_BUDGET : P4DESK_TYPEFACE_UNAVAILABLE;
        }
    }
    xSemaphoreGive(s_mutex);
    return result;
}

bool p4desk_typeface_ready(void)
{
    if (!lock_face()) {
        return false;
    }
    const bool ready = s_face != NULL;
    xSemaphoreGive(s_mutex);
    return ready;
}

static bool valid_request(uint32_t codepoint, uint16_t px)
{
    return codepoint <= 0x10ffffu && !(codepoint >= 0xd800u && codepoint <= 0xdfffu)
        && px >= 8u && px <= 128u;
}

static int32_t load_glyph_locked(uint32_t codepoint, uint16_t px,
                                 p4desk_typeface_metrics_t *out)
{
    if (s_face == NULL) {
        return P4DESK_TYPEFACE_UNAVAILABLE;
    }
    if (!s_glyph_valid || s_glyph_codepoint != codepoint || s_glyph_px != px) {
        s_glyph_valid = false;
        s_glyph_rendered = false;
        FT_UInt index = FT_Get_Char_Index(s_face, codepoint);
        if (index == 0) {
            return P4DESK_TYPEFACE_MISSING;
        }
        const uint32_t before_failures = s_memory.allocation_failures;
        if (FT_Set_Pixel_Sizes(s_face, 0, px) != 0
            || FT_Load_Glyph(s_face, index, FT_LOAD_NO_BITMAP | FT_LOAD_TARGET_NORMAL) != 0) {
            return s_memory.allocation_failures != before_failures
                ? P4DESK_TYPEFACE_BUDGET : P4DESK_TYPEFACE_RENDER_FAILED;
        }
        s_glyph_codepoint = codepoint;
        s_glyph_px = px;
        const FT_GlyphSlot glyph = s_face->glyph;
        if (glyph->format != FT_GLYPH_FORMAT_OUTLINE) {
            return P4DESK_TYPEFACE_RENDER_FAILED;
        }
        FT_BBox bounds;
        FT_Outline_Get_CBox(&glyph->outline, &bounds);
        /* The normal smooth renderer uses the outline control box rounded
         * outward to the pixel grid. Do the same without rasterizing pixels. */
        const int64_t x0 = (int64_t)bounds.xMin / 64 - (bounds.xMin < 0 && bounds.xMin % 64 != 0);
        const int64_t y0 = (int64_t)bounds.yMin / 64 - (bounds.yMin < 0 && bounds.yMin % 64 != 0);
        const int64_t x1 = (int64_t)bounds.xMax / 64 + (bounds.xMax > 0 && bounds.xMax % 64 != 0);
        const int64_t y1 = (int64_t)bounds.yMax / 64 + (bounds.yMax > 0 && bounds.yMax % 64 != 0);
        const float advance = (float)glyph->advance.x / 64.0f;
        if (x0 < INT16_MIN || x0 > INT16_MAX || y0 < INT16_MIN || y0 > INT16_MAX
            || x1 < x0 || y1 < y0 || x1 - x0 > 256 || y1 - y0 > 256
            || !isfinite(advance) || advance < 0.0f || advance > 512.0f) {
            return P4DESK_TYPEFACE_RENDER_FAILED;
        }
        s_glyph_metrics = (p4desk_typeface_metrics_t){
            .width = (uint16_t)(x1 - x0), .height = (uint16_t)(y1 - y0),
            .xmin = (int16_t)x0, .ymin = (int16_t)y0, .advance = advance,
        };
        s_glyph_valid = true;
    }
    *out = s_glyph_metrics;
    return P4DESK_TYPEFACE_OK;
}

static int32_t render_glyph_locked(void)
{
    if (!s_glyph_rendered) {
        const uint32_t before_failures = s_memory.allocation_failures;
        if (FT_Render_Glyph(s_face->glyph, FT_RENDER_MODE_NORMAL) != 0) {
            s_glyph_valid = false;
            return s_memory.allocation_failures != before_failures
                ? P4DESK_TYPEFACE_BUDGET : P4DESK_TYPEFACE_RENDER_FAILED;
        }
        s_glyph_rendered = true;
    }
    const FT_GlyphSlot glyph = s_face->glyph;
    const FT_Bitmap *bitmap = &glyph->bitmap;
    const int64_t bottom = (int64_t)glyph->bitmap_top - bitmap->rows;
    const int64_t pitch = bitmap->pitch < 0 ? -(int64_t)bitmap->pitch : bitmap->pitch;
    const float advance = (float)glyph->advance.x / 64.0f;
    if (bitmap->width > 256u || bitmap->rows > 256u || glyph->bitmap_left < INT16_MIN
        || glyph->bitmap_left > INT16_MAX || bottom < INT16_MIN || bottom > INT16_MAX
        || !isfinite(advance) || advance < 0.0f || advance > 512.0f
        || (bitmap->width != 0 && bitmap->rows != 0 && (bitmap->buffer == NULL
            || bitmap->pixel_mode != FT_PIXEL_MODE_GRAY || bitmap->num_grays < 2u
            || bitmap->num_grays > 256u || pitch < bitmap->width
            || (uint64_t)pitch * bitmap->rows > TYPEFACE_MEMORY_LIMIT))) {
        s_glyph_valid = false;
        return P4DESK_TYPEFACE_RENDER_FAILED;
    }
    if (bitmap->width != s_glyph_metrics.width || bitmap->rows != s_glyph_metrics.height
        || glyph->bitmap_left != s_glyph_metrics.xmin || bottom != s_glyph_metrics.ymin
        || advance != s_glyph_metrics.advance) {
        s_glyph_valid = false;
        return P4DESK_TYPEFACE_RENDER_FAILED;
    }
    return P4DESK_TYPEFACE_OK;
}

int32_t p4desk_typeface_metrics(uint32_t codepoint, uint16_t px,
                               p4desk_typeface_metrics_t *out)
{
    if (out == NULL) {
        return P4DESK_TYPEFACE_INVALID;
    }
    memset(out, 0, sizeof(*out));
    if (!valid_request(codepoint, px)) {
        return P4DESK_TYPEFACE_INVALID;
    }
    if (!lock_face()) {
        return P4DESK_TYPEFACE_UNAVAILABLE;
    }
    const int32_t result = load_glyph_locked(codepoint, px, out);
    xSemaphoreGive(s_mutex);
    return result;
}

int32_t p4desk_typeface_render(uint32_t codepoint, uint16_t px,
                              uint8_t *alpha, size_t capacity,
                              p4desk_typeface_metrics_t *out)
{
    if (out == NULL) {
        return P4DESK_TYPEFACE_INVALID;
    }
    memset(out, 0, sizeof(*out));
    if (!valid_request(codepoint, px)) {
        return P4DESK_TYPEFACE_INVALID;
    }
    if (!lock_face()) {
        return P4DESK_TYPEFACE_UNAVAILABLE;
    }
    p4desk_typeface_metrics_t metrics;
    int32_t result = load_glyph_locked(codepoint, px, &metrics);
    if (result == P4DESK_TYPEFACE_OK) {
        const size_t required = (size_t)metrics.width * metrics.height;
        if (capacity < required || (required != 0 && alpha == NULL)) {
            result = P4DESK_TYPEFACE_BUFFER_SMALL;
        } else if ((result = render_glyph_locked()) != P4DESK_TYPEFACE_OK) {
            /* Preserve zeroed output and leave the caller's buffer untouched. */
        } else {
            const FT_Bitmap *bitmap = &s_face->glyph->bitmap;
            for (size_t y = 0; y < metrics.height && metrics.width != 0; ++y) {
                /* A negative pitch stores the bottom row at the beginning of
                 * the allocation. Normalize it to top-down tight alpha8. */
                const size_t source_y = bitmap->pitch < 0 ? metrics.height - 1u - y : y;
                const size_t stride = bitmap->pitch < 0 ? (size_t)-(int64_t)bitmap->pitch
                                                       : (size_t)bitmap->pitch;
                const uint8_t *row = bitmap->buffer + source_y * stride;
                uint8_t *target = alpha + y * metrics.width;
                if (bitmap->num_grays == 256u) {
                    memcpy(target, row, metrics.width);
                } else {
                    const uint32_t maximum = bitmap->num_grays - 1u;
                    for (size_t x = 0; x < metrics.width; ++x) {
                        const uint32_t value = row[x] > maximum ? maximum : row[x];
                        target[x] = (uint8_t)((value * 255u + maximum / 2u) / maximum);
                    }
                }
            }
            *out = metrics;
        }
    }
    xSemaphoreGive(s_mutex);
    return result;
}

void p4desk_typeface_memory(p4desk_typeface_memory_t *out)
{
    if (out == NULL) {
        return;
    }
    memset(out, 0, sizeof(*out));
    if (lock_face()) {
        *out = s_memory;
        xSemaphoreGive(s_mutex);
    }
}
