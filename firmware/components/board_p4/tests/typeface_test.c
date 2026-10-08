/* Tests production HAL against real FreeType public headers and fake FT calls. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

struct typeface_test_stat { mode_t st_mode; off_t st_size; };
static int typeface_test_stat(const char *path, struct typeface_test_stat *out);
#define stat typeface_test_stat
#include "../typeface.c"
#undef stat

static int s_font_exists;
static int s_ft_open_fail;
static int s_force_budget;
static unsigned s_opens;
static unsigned s_closes;
static unsigned s_loads;
static unsigned s_renders;
static unsigned s_heap_fail;
static void *s_library_block;
static void *s_face_block;
static FT_FaceRec s_fake_face;
static FT_GlyphSlotRec s_fake_glyph;
static unsigned char s_mask[] = { 3, 7, 0, 9, 15, 0 };

static int typeface_test_stat(const char *path, struct typeface_test_stat *out)
{
    assert(strcmp(path, "/sdcard/fonts/HarmonyOS_Sans_SC_Regular.ttf") == 0
        || strcmp(path, "/sdcard/fonts/NotoSansCJKsc-Regular.otf") == 0
        || strcmp(path, "/sdcard/typeface/HarmonyOS_Sans_SC_Regular.ttf") == 0);
    if (!s_font_exists || strcmp(path, "/sdcard/fonts/NotoSansCJKsc-Regular.otf") != 0) {
        return -1;
    }
    *out = (struct typeface_test_stat){ .st_mode = S_IFREG, .st_size = 16400000 };
    return 0;
}

void *heap_caps_malloc(size_t size, unsigned caps)
{
    assert(caps == (MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT));
    return s_heap_fail ? NULL : malloc(size);
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

FT_Error FT_New_Library(FT_Memory memory, FT_Library *library)
{
    s_library_block = memory->alloc(memory, 64);
    if (s_library_block == NULL) { *library = NULL; return 1; }
    *library = (FT_Library)s_library_block;
    return 0;
}
void FT_Add_Default_Modules(FT_Library library) { assert(library != NULL); }
FT_Error FT_Done_Library(FT_Library library)
{
    assert(library == (FT_Library)s_library_block);
    s_ft_memory.free(&s_ft_memory, s_library_block);
    s_library_block = NULL;
    return 0;
}
FT_Error FT_New_Face(FT_Library library, const char *path, FT_Long index, FT_Face *face)
{
    assert(library != NULL && index == 0);
    assert(strcmp(path, "/sdcard/fonts/NotoSansCJKsc-Regular.otf") == 0);
    ++s_opens;
    if (s_ft_open_fail) { *face = NULL; return 1; }
    s_face_block = s_ft_memory.alloc(&s_ft_memory, s_force_budget ? TYPEFACE_MEMORY_LIMIT : 256);
    if (s_face_block == NULL) { *face = NULL; return 1; }
    s_fake_face.face_flags = FT_FACE_FLAG_SCALABLE;
    s_fake_face.glyph = &s_fake_glyph;
    *face = &s_fake_face;
    return 0;
}
FT_Error FT_Done_Face(FT_Face face)
{
    assert(face == &s_fake_face);
    ++s_closes;
    s_ft_memory.free(&s_ft_memory, s_face_block);
    s_face_block = NULL;
    return 0;
}
FT_Error FT_Select_Charmap(FT_Face face, FT_Encoding encoding)
{
    assert(face == &s_fake_face && encoding == FT_ENCODING_UNICODE);
    return 0;
}
FT_UInt FT_Get_Char_Index(FT_Face face, FT_ULong character)
{
    assert(face == &s_fake_face);
    return character == 0x10ffff ? 0 : 1;
}
FT_Error FT_Set_Pixel_Sizes(FT_Face face, FT_UInt width, FT_UInt height)
{
    assert(face == &s_fake_face && width == 0 && height >= 8 && height <= 128);
    return 0;
}
FT_Error FT_Load_Glyph(FT_Face face, FT_UInt index, FT_Int32 flags)
{
    assert(face == &s_fake_face && index == 1);
    assert(flags == (FT_LOAD_NO_BITMAP | FT_LOAD_TARGET_NORMAL));
    ++s_loads;
    s_fake_glyph.bitmap = (FT_Bitmap){ .width = 2, .rows = 2, .pitch = 3,
        .buffer = s_mask, .num_grays = 256, .pixel_mode = FT_PIXEL_MODE_GRAY };
    s_fake_glyph.bitmap_left = -1;
    s_fake_glyph.bitmap_top = 2;
    s_fake_glyph.advance.x = 18 * 64;
    s_fake_glyph.format = FT_GLYPH_FORMAT_OUTLINE;
    return 0;
}
void FT_Outline_Get_CBox(const FT_Outline *outline, FT_BBox *bounds)
{
    assert(outline == &s_fake_glyph.outline);
    *bounds = (FT_BBox){ .xMin = -63, .xMax = 63, .yMin = 0, .yMax = 128 };
}
FT_Error FT_Render_Glyph(FT_GlyphSlot glyph, FT_Render_Mode mode)
{
    assert(glyph == &s_fake_glyph && mode == FT_RENDER_MODE_NORMAL);
    ++s_renders;
    return 0;
}

static void assert_zero_metrics(p4desk_typeface_metrics_t metrics)
{
    assert(metrics.width == 0 && metrics.height == 0 && metrics.xmin == 0
        && metrics.ymin == 0 && metrics.advance == 0);
}

int main(void)
{
    p4desk_typeface_metrics_t metrics;
    unsigned char output[4];
    assert(!p4desk_typeface_ready());
    assert(p4desk_typeface_metrics(0x4e00, 18, &metrics) == P4DESK_TYPEFACE_UNAVAILABLE);
    assert_zero_metrics(metrics);
    assert(s_opens == 0); /* UI measurement never opens a font. */
    assert(p4desk_typeface_init() == P4DESK_TYPEFACE_UNAVAILABLE);
    assert(s_memory.current_bytes == 0);
    s_font_exists = 1;
    s_ft_open_fail = 1;
    assert(p4desk_typeface_init() == P4DESK_TYPEFACE_UNAVAILABLE);
    assert(s_memory.current_bytes == 0);
    s_ft_open_fail = 0;
    s_force_budget = 1;
    assert(p4desk_typeface_init() == P4DESK_TYPEFACE_BUDGET);
    assert(s_memory.current_bytes == 0);
    s_force_budget = 0;
    assert(p4desk_typeface_init() == P4DESK_TYPEFACE_OK);
    assert(p4desk_typeface_ready());
    unsigned opens = s_opens;
    assert(p4desk_typeface_init() == P4DESK_TYPEFACE_OK && s_opens == opens);

    assert(p4desk_typeface_metrics(0x110000, 18, &metrics) == P4DESK_TYPEFACE_INVALID);
    assert(p4desk_typeface_metrics(0xd800, 18, &metrics) == P4DESK_TYPEFACE_INVALID);
    assert(p4desk_typeface_metrics(0x4e00, 7, &metrics) == P4DESK_TYPEFACE_INVALID);
    assert(p4desk_typeface_metrics(0x4e00, 129, &metrics) == P4DESK_TYPEFACE_INVALID);
    assert(p4desk_typeface_metrics(0x4e00, 18, NULL) == P4DESK_TYPEFACE_INVALID);
    assert(p4desk_typeface_metrics(0x10ffff, 18, &metrics) == P4DESK_TYPEFACE_MISSING);
    assert_zero_metrics(metrics);
    assert(p4desk_typeface_metrics(0x4e00, 18, &metrics) == P4DESK_TYPEFACE_OK);
    assert(metrics.width == 2 && metrics.height == 2 && metrics.xmin == -1
        && metrics.ymin == 0 && metrics.advance == 18);
    assert(s_renders == 0); /* Measuring loads outlines, never alpha bitmaps. */
    unsigned loads = s_loads;
    assert(p4desk_typeface_render(0x4e00, 18, output, 3, &metrics) == P4DESK_TYPEFACE_BUFFER_SMALL);
    assert_zero_metrics(metrics);
    assert(p4desk_typeface_render(0x4e00, 18, NULL, 4, &metrics) == P4DESK_TYPEFACE_BUFFER_SMALL);
    assert(s_renders == 0); /* Insufficient buffer does not rasterize either. */
    assert(p4desk_typeface_render(0x4e00, 18, output, 4, &metrics) == P4DESK_TYPEFACE_OK);
    assert(memcmp(output, (unsigned char[]){3, 7, 9, 15}, 4) == 0 && s_loads == loads);
    assert(s_renders == 1);
    s_fake_glyph.bitmap.pitch = -3;
    assert(p4desk_typeface_render(0x4e00, 18, output, 4, &metrics) == P4DESK_TYPEFACE_OK);
    assert(memcmp(output, (unsigned char[]){9, 15, 3, 7}, 4) == 0);
    s_fake_glyph.bitmap.pitch = 3;
    s_fake_glyph.bitmap.num_grays = 16;
    assert(p4desk_typeface_render(0x4e00, 18, output, 4, &metrics) == P4DESK_TYPEFACE_OK);
    assert(memcmp(output, (unsigned char[]){51, 119, 153, 255}, 4) == 0);
    s_fake_glyph.bitmap.pixel_mode = FT_PIXEL_MODE_BGRA;
    assert(p4desk_typeface_render(0x4e00, 18, output, 4, &metrics) == P4DESK_TYPEFACE_RENDER_FAILED);
    assert_zero_metrics(metrics);
    assert(p4desk_typeface_metrics(0x4e00, 18, &metrics) == P4DESK_TYPEFACE_OK);
    s_fake_glyph.bitmap.width = 257;
    assert(p4desk_typeface_render(0x4e00, 18, output, 4, &metrics) == P4DESK_TYPEFACE_RENDER_FAILED);

    size_t baseline = s_memory.current_bytes;
    assert(ft_allocate(&s_ft_memory, 0) == NULL);
    assert(ft_allocate(&s_ft_memory, LONG_MAX) == NULL);
    void *small = ft_allocate(&s_ft_memory, 128);
    assert(small != NULL && (uintptr_t)small % _Alignof(max_align_t) == 0);
    memset(small, 42, 128);
    assert(ft_reallocate(&s_ft_memory, 128, 64, small) == small);
    void *large = ft_reallocate(&s_ft_memory, 64, 256, small);
    assert(large != NULL && ((unsigned char *)large)[127] == 42);
    ft_release(&s_ft_memory, large);
    assert(s_memory.current_bytes == baseline);
    s_heap_fail = 1;
    assert(ft_allocate(&s_ft_memory, 32) == NULL);
    s_heap_fail = 0;
    void *budget = ft_allocate(&s_ft_memory, 1400000);
    assert(budget != NULL);
    assert(ft_reallocate(&s_ft_memory, 1400000, 1500000, budget) == NULL);
    assert(s_memory.current_bytes <= TYPEFACE_MEMORY_LIMIT);
    ft_release(&s_ft_memory, budget);
    assert(s_memory.current_bytes == baseline);
    p4desk_typeface_memory_t stats;
    p4desk_typeface_memory(&stats);
    assert(stats.current_bytes == baseline && stats.peak_bytes <= TYPEFACE_MEMORY_LIMIT);
    assert(stats.allocation_failures >= 5 && stats.successful_allocations > 0);
    assert(lock_face());
    close_font_locked();
    xSemaphoreGive(s_mutex);
    assert(s_memory.current_bytes == 0 && s_closes == 1);
    assert(!p4desk_typeface_ready());
    puts("typeface: initialization/retry, FFI, alpha/pitch, bounds and PSRAM budget passed");
    return 0;
}
