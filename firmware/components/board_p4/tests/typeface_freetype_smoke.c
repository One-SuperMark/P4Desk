/* Production HAL with real FreeType and fixed synthetic Unicode samples.
 * Host filesystem names are redirected to an explicit test font argument. */
#include <assert.h>
#include <stdio.h>
#include <sys/stat.h>
#include "ft2build.h"
#include FT_FREETYPE_H

static const char *s_test_font;
struct typeface_smoke_stat { mode_t st_mode; off_t st_size; };
static int typeface_smoke_stat(const char *path, struct typeface_smoke_stat *out)
{
    (void)path;
    struct stat status;
    if (stat(s_test_font, &status) != 0) { return -1; }
    out->st_mode = status.st_mode;
    out->st_size = status.st_size;
    return 0;
}
static FT_Error typeface_smoke_new_face(FT_Library library, const char *path,
                                        FT_Long index, FT_Face *face)
{
    (void)path;
    return FT_New_Face(library, s_test_font, index, face);
}
#define stat typeface_smoke_stat
#define FT_New_Face typeface_smoke_new_face
#include "../typeface.c"
#undef FT_New_Face
#undef stat

void *heap_caps_malloc(size_t size, unsigned caps)
{
    assert(caps == (MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT));
    return malloc(size);
}
void heap_caps_free(void *block) { free(block); }
SemaphoreHandle_t xSemaphoreCreateMutexStatic(StaticSemaphore_t *storage)
{
    assert(pthread_mutex_init(&storage->lock, NULL) == 0);
    return storage;
}
int xSemaphoreTake(SemaphoreHandle_t mutex, unsigned timeout)
{
    assert(timeout == TYPEFACE_LOCK_MS);
    return pthread_mutex_lock(&mutex->lock) == 0;
}
int xSemaphoreGive(SemaphoreHandle_t mutex) { return pthread_mutex_unlock(&mutex->lock) == 0; }

int main(int argc, char **argv)
{
    assert(argc == 2);
    s_test_font = argv[1];
    assert(p4desk_typeface_init() == P4DESK_TYPEFACE_OK);
    uint8_t bitmap[256 * 256];
    const uint16_t sizes[] = {14, 18, 22, 24};
    size_t rendered = 0;
    uint32_t first_cycle_bytes = 0;
    for (size_t cycle = 0; cycle < 2; ++cycle) {
        for (size_t size = 0; size < sizeof(sizes) / sizeof(sizes[0]); ++size) {
            for (uint32_t cp = 0x4e00; cp <= 0x9fff; cp += 17) {
                p4desk_typeface_metrics_t a, b;
                int32_t result = p4desk_typeface_metrics(cp, sizes[size], &a);
                if (result == P4DESK_TYPEFACE_MISSING) { continue; }
                assert(result == P4DESK_TYPEFACE_OK);
                assert(p4desk_typeface_render(cp, sizes[size], bitmap, sizeof(bitmap), &b) == P4DESK_TYPEFACE_OK);
                assert(memcmp(&a, &b, sizeof(a)) == 0);
                ++rendered;
            }
        }
        p4desk_typeface_memory_t cycle_stats;
        p4desk_typeface_memory(&cycle_stats);
        if (cycle == 0) { first_cycle_bytes = cycle_stats.current_bytes; }
        else { assert(cycle_stats.current_bytes == first_cycle_bytes); }
    }
    p4desk_typeface_memory_t stats;
    p4desk_typeface_memory(&stats);
    assert(stats.current_bytes <= TYPEFACE_MEMORY_LIMIT && stats.peak_bytes <= TYPEFACE_MEMORY_LIMIT);
    assert(stats.allocation_failures == 0 && rendered > 0);
    printf("typeface real font: rendered=%zu current=%u peak=%u failures=%u repeated_cycle_growth=0\n",
           rendered, stats.current_bytes, stats.peak_bytes, stats.allocation_failures);
    assert(lock_face());
    close_font_locked();
    xSemaphoreGive(s_mutex);
    assert(s_memory.current_bytes == 0);
    return 0;
}
