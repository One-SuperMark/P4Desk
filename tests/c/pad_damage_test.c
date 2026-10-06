#include "pad_damage.h"
#include "display_pipeline.h"

#include <assert.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void expect_rect(p4pad_rect_t rect, int32_t x1, int32_t y1, int32_t x2, int32_t y2)
{
    assert(rect.x1 == x1 && rect.y1 == y1 && rect.x2 == x2 && rect.y2 == y2);
}

static void independent_full_compare(const uint16_t *destination, size_t destination_stride,
                                     const uint16_t *source, size_t source_stride,
                                     unsigned width, unsigned height, bool rotate)
{
    // This reference uses inverse destination coordinates, independently of the
    // module's source-row loop and rectangle mapping.
    for (unsigned y = 0; y < height; ++y) {
        for (unsigned x = 0; x < width; ++x) {
            const unsigned sy = rotate ? height - 1 - y : y;
            const unsigned sx = rotate ? width - 1 - x : x;
            assert(destination[(size_t)y * destination_stride + x] ==
                   source[(size_t)sy * source_stride + sx]);
        }
    }
}

static void check_padding(const uint16_t *destination, unsigned width, unsigned height,
                          size_t stride, unsigned allocation_height, uint16_t guard)
{
    for (unsigned y = 0; y < height; ++y)
        for (size_t x = width; x < stride; ++x) assert(destination[(size_t)y * stride + x] == guard);
    for (unsigned y = height; y < allocation_height; ++y)
        for (size_t x = 0; x < stride; ++x) assert(destination[(size_t)y * stride + x] == guard);
}

static void debt_is_independent_clipped_and_unioned(void)
{
    p4pad_damage_t damage;
    p4pad_rect_t rect;
    assert(p4pad_damage_init(&damage, 17, 11));
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        assert(p4pad_damage_peek(&damage, i, &rect));
        expect_rect(rect, 0, 0, 17, 11);
        assert(p4pad_damage_take(&damage, i, &rect));
        expect_rect(rect, 0, 0, 17, 11);
        assert(!p4pad_damage_take(&damage, i, &rect));
    }
    assert(p4pad_damage_mark(&damage, -8, -4, 5, 3));
    assert(p4pad_damage_take(&damage, 0, &rect));
    expect_rect(rect, 0, 0, 5, 3);
    assert(p4pad_damage_mark(&damage, 12, 8, 30, 20));
    assert(p4pad_damage_peek(&damage, 0, &rect));
    expect_rect(rect, 12, 8, 17, 11);
    for (unsigned i = 1; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        assert(p4pad_damage_peek(&damage, i, &rect));
        expect_rect(rect, 0, 0, 17, 11);
    }
    assert(!p4pad_damage_mark(&damage, 4, 4, 4, 9));
    assert(!p4pad_damage_mark(&damage, 8, 8, 7, 9));
    assert(!p4pad_damage_mark(&damage, 20, 8, 24, 9));
    assert(!p4pad_damage_mark(&damage, -9, -4, -1, 8));
    assert(p4pad_damage_peek(&damage, 0, &rect));
    expect_rect(rect, 12, 8, 17, 11);
    p4pad_damage_invalidate_all(&damage);
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        assert(p4pad_damage_take(&damage, i, &rect));
        expect_rect(rect, 0, 0, 17, 11);
    }
    assert(!p4pad_damage_take(&damage, P4PAD_DAMAGE_BUFFER_COUNT, &rect));
    expect_rect(rect, 0, 0, 0, 0);
    assert(!p4pad_damage_init(&damage, 0, 11));
    assert(!p4pad_damage_mark(&damage, 0, 0, 1, 1));
    assert(!p4pad_damage_init(&damage, UINT32_MAX, 11));
    assert(!p4pad_damage_init(NULL, 17, 11));
}

static void rotation_and_row_spans_are_exact(void)
{
    p4pad_damage_t damage;
    p4pad_rect_t rect = {2, 3, 7, 9}, mapped;
    size_t first, past_last;
    assert(p4pad_damage_init(&damage, 17, 11));
    assert(p4pad_damage_map_rect(&damage, &rect, false, &mapped));
    expect_rect(mapped, 2, 3, 7, 9);
    assert(p4pad_damage_row_span(&damage, &rect, false, 22, &first, &past_last));
    assert(first == 3 * 22 + 2 && past_last == 8 * 22 + 7);
    assert(p4pad_damage_map_rect(&damage, &rect, true, &mapped));
    expect_rect(mapped, 10, 2, 15, 8);
    assert(p4pad_damage_row_span(&damage, &rect, true, 22, &first, &past_last));
    assert(first == 2 * 22 + 10 && past_last == 7 * 22 + 15);
    assert(p4pad_damage_map_rect(&damage, &mapped, true, &mapped));
    expect_rect(mapped, 2, 3, 7, 9);
    rect = (p4pad_rect_t){16, 10, 17, 11};
    assert(p4pad_damage_row_span(&damage, &rect, true, 22, &first, &past_last));
    assert(first == 0 && past_last == 1);
    assert(!p4pad_damage_row_span(&damage, &rect, true, 16, &first, &past_last));
    assert(!p4pad_damage_row_span(&damage, &rect, true, SIZE_MAX, &first, &past_last));
    rect.x1 = -1;
    assert(!p4pad_damage_map_rect(&damage, &rect, true, &mapped));
}

static void distant_changes_do_not_copy_the_space_between_them(void)
{
    p4pad_damage_t damage;
    p4pad_damage_batch_t batch;
    p4pad_rect_t rect;
    assert(p4pad_damage_init(&damage, 1024, 600));
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        assert(p4pad_damage_take_batch(&damage, i, &batch));
        assert(batch.count == 1 && batch.pixels == 1024U * 600U);
        expect_rect(batch.regions[0], 0, 0, 1024, 600);
    }
    assert(p4pad_damage_mark(&damage, 16, 24, 64, 40));
    assert(p4pad_damage_mark(&damage, 760, 556, 832, 572));
    assert(!p4pad_damage_take_batch(&damage, 0, NULL));
    assert(p4pad_damage_take_batch(&damage, 0, &batch));
    assert(batch.count == 2 && batch.pixels == 1920);
    expect_rect(batch.bounds, 16, 24, 832, 572);
    expect_rect(batch.regions[0], 16, 24, 64, 40);
    expect_rect(batch.regions[1], 760, 556, 832, 572);
    assert(p4pad_damage_mark(&damage, 128, 200, 148, 220));
    assert(p4pad_damage_take_batch(&damage, 0, &batch));
    assert(batch.count == 1 && batch.pixels == 400);
    assert(p4pad_damage_take_batch(&damage, 2, &batch));
    assert(batch.count == 3 && batch.pixels == 2320);
    // Legacy bounding take and batch take share one history, never two debts.
    assert(p4pad_damage_take(&damage, 1, &rect));
    assert(!p4pad_damage_take_batch(&damage, 1, &batch));
    assert(batch.count == 0 && batch.pixels == 0);
    assert(!p4pad_damage_take_batch(&damage, P4PAD_DAMAGE_BUFFER_COUNT, &batch));
    damage.band_height = 1;
    assert(!p4pad_damage_mark(&damage, 0, 0, 1, 600));
    assert(!p4pad_damage_take_batch(&damage, 0, &batch));
}

static void small_copy_preserves_every_untouched_pixel(bool rotate)
{
    enum { WIDTH = 17, HEIGHT = 11, SS = 22, DS = 24 };
    uint16_t source[SS * HEIGHT], destination[DS * HEIGHT];
    for (unsigned i = 0; i < SS * HEIGHT; ++i) source[i] = (uint16_t)(i + 1);
    for (unsigned i = 0; i < DS * HEIGHT; ++i) destination[i] = 0xabcd;
    p4pad_damage_t damage;
    p4pad_rect_t rect = {2, 3, 7, 9};
    assert(p4pad_damage_init(&damage, WIDTH, HEIGHT));
    assert(p4pad_damage_copy_rgb565(&damage, destination, DS * HEIGHT, DS,
                                  source, SS * HEIGHT, SS, &rect, rotate));
    for (unsigned y = 0; y < HEIGHT; ++y) {
        for (unsigned x = 0; x < DS; ++x) {
            unsigned sy = rotate ? HEIGHT - 1 - y : y;
            unsigned sx = x < WIDTH && rotate ? WIDTH - 1 - x : x;
            const bool touched = x < WIDTH && sx >= 2 && sx < 7 && sy >= 3 && sy < 9;
            assert(destination[y * DS + x] == (touched ? source[sy * SS + sx] : 0xabcd));
        }
    }
    for (unsigned i = 0; i < SS * HEIGHT; ++i) assert(source[i] == (uint16_t)(i + 1));
    assert(!p4pad_damage_copy_rgb565(&damage, source, SS * HEIGHT, SS,
                                   source, SS * HEIGHT, SS, &rect, rotate));
    assert(!p4pad_damage_copy_rgb565(&damage, source + 1, SS * HEIGHT - 1, WIDTH,
                                   source, SS * HEIGHT, SS, &rect, rotate));
    assert(!p4pad_damage_copy_rgb565(&damage, destination, WIDTH, DS,
                                   source, SS * HEIGHT, SS, &rect, rotate));
    assert(!p4pad_damage_copy_rgb565(&damage, destination, DS * HEIGHT, DS,
                                   source, WIDTH, SS, &rect, rotate));
    assert(!p4pad_damage_copy_rgb565(&damage, destination, DS * HEIGHT, WIDTH - 1,
                                   source, SS * HEIGHT, SS, &rect, rotate));
    assert(!p4pad_damage_copy_rgb565(&damage, destination, SIZE_MAX, SIZE_MAX,
                                   source, SS * HEIGHT, SS, &rect, rotate));
    assert(!p4pad_damage_copy_rgb565(&damage, NULL, DS * HEIGHT, DS,
                                   source, SS * HEIGHT, SS, &rect, rotate));
}

static void prepare_and_compare(p4pad_damage_t *damage, unsigned index, uint16_t *destination,
                                size_t destination_count, size_t destination_stride,
                                const uint16_t *source, size_t source_count, size_t source_stride,
                                bool rotate)
{
    p4pad_rect_t rect;
    p4pad_damage_batch_t batch;
    assert(p4pad_damage_take_batch(damage, index, &batch));
    assert(batch.count > 0 && batch.count <= P4PAD_DAMAGE_BANDS);
    size_t pixels = 0;
    for (size_t i = 0; i < batch.count; ++i) {
        pixels += (size_t)(batch.regions[i].x2 - batch.regions[i].x1) *
                  (size_t)(batch.regions[i].y2 - batch.regions[i].y1);
        if (i) assert(batch.regions[i - 1].y2 <= batch.regions[i].y1);
        assert(p4pad_damage_copy_rgb565(damage, destination, destination_count, destination_stride,
                                      source, source_count, source_stride, &batch.regions[i], rotate));
    }
    assert(pixels == batch.pixels && pixels <= (size_t)damage->width * damage->height);
    independent_full_compare(destination, destination_stride, source, source_stride,
                             (unsigned)damage->width, (unsigned)damage->height, rotate);
    assert(!p4pad_damage_peek(damage, index, &rect));
}

static void three_buffers_catch_up_to_all_source_updates(unsigned width, unsigned height, bool rotate)
{
    const size_t source_stride = width + 3;
    const size_t destination_stride = width + 5;
    const unsigned allocation_height = height + 8; // MCU row padding must survive.
    const size_t source_count = source_stride * allocation_height;
    const size_t destination_count = destination_stride * allocation_height;
    uint16_t *source = malloc(source_count * sizeof(uint16_t));
    uint16_t *destination[P4PAD_DAMAGE_BUFFER_COUNT];
    assert(source);
    for (size_t i = 0; i < source_count; ++i) source[i] = (uint16_t)(i * 31 + 7);
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        destination[i] = malloc(destination_count * sizeof(uint16_t));
        assert(destination[i]);
        for (size_t j = 0; j < destination_count; ++j) destination[i][j] = 0xcafe;
    }
    p4pad_damage_t damage;
    assert(p4pad_damage_init(&damage, width, height));
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i)
        prepare_and_compare(&damage, i, destination[i], destination_count, destination_stride,
                            source, source_count, source_stride, rotate);
    for (unsigned frame = 0; frame < 120; ++frame) {
        // Several changes between preparations; some buffers remain stale for
        // many frames. Their own union must retain every missed source update.
        for (unsigned update = 0; update < 3; ++update) {
            unsigned x1 = (frame * 19 + update * 7) % width;
            unsigned y1 = (frame * 11 + update * 3) % height;
            unsigned x2 = x1 + 1 + (frame + update) % 9;
            unsigned y2 = y1 + 1 + (frame + update) % 5;
            if (x2 > width) x2 = width;
            if (y2 > height) y2 = height;
            for (unsigned y = y1; y < y2; ++y)
                for (unsigned x = x1; x < x2; ++x)
                    source[(size_t)y * source_stride + x] = (uint16_t)(frame * 101 + update * 47 + x + y);
            assert(p4pad_damage_mark(&damage, (int32_t)x1, (int32_t)y1, (int32_t)x2, (int32_t)y2));
        }
        unsigned index = frame % 13 == 0 ? 2 : frame % 2;
        prepare_and_compare(&damage, index, destination[index], destination_count, destination_stride,
                            source, source_count, source_stride, rotate);
        for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i)
            check_padding(destination[i], width, height, destination_stride, allocation_height, 0xcafe);
    }
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        p4pad_rect_t rect;
        if (p4pad_damage_peek(&damage, i, &rect))
            prepare_and_compare(&damage, i, destination[i], destination_count, destination_stride,
                                source, source_count, source_stride, rotate);
        independent_full_compare(destination[i], destination_stride, source, source_stride, width, height, rotate);
    }
    // USB/display mode may have overwritten visible framebuffer pixels. A
    // return to Pad invalidates every buffer, even with no new Pad writes.
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i)
        for (unsigned y = 0; y < height; ++y)
            for (unsigned x = 0; x < width; ++x) destination[i][(size_t)y * destination_stride + x] = 0x1234;
    p4pad_damage_invalidate_all(&damage);
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        p4pad_rect_t rect;
        assert(p4pad_damage_peek(&damage, i, &rect));
        expect_rect(rect, 0, 0, (int32_t)width, (int32_t)height);
        prepare_and_compare(&damage, i, destination[i], destination_count, destination_stride,
                            source, source_count, source_stride, rotate);
        check_padding(destination[i], width, height, destination_stride, allocation_height, 0xcafe);
        free(destination[i]);
    }
    free(source);
}

static void stale_job_cancellation_restores_unpublished_debt(bool rotate, bool reached_ready)
{
    enum { WIDTH = 17, HEIGHT = 11, STRIDE = 19 };
    uint16_t source[STRIDE * HEIGHT];
    uint16_t cpu[P4PAD_DAMAGE_BUFFER_COUNT][STRIDE * HEIGHT];
    uint16_t published[P4PAD_DAMAGE_BUFFER_COUNT][STRIDE * HEIGHT];
    for (unsigned i = 0; i < STRIDE * HEIGHT; ++i) source[i] = (uint16_t)(i * 31 + 7);
    p4pad_damage_t damage;
    p4dp_owner_t owner;
    p4pad_rect_t rect;
    assert(p4pad_damage_init(&damage, WIDTH, HEIGHT));
    p4dp_init(&owner, 0);
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        for (unsigned j = 0; j < STRIDE * HEIGHT; ++j) cpu[i][j] = published[i][j] = 0xcafe;
        prepare_and_compare(&damage, i, cpu[i], STRIDE * HEIGHT, STRIDE,
                            source, STRIDE * HEIGHT, STRIDE, rotate);
        memcpy(published[i], cpu[i], sizeof(cpu[i]));
    }

    // A Pad job holds an old epoch before acquiring the source lock. While it
    // waits, USB writes the LCD buffers and a newer Pad epoch requests a full
    // restore. That old job then consumes the newer restoration's debt.
    const unsigned stale_epoch = 1, current_epoch = 3;
    assert(stale_epoch != current_epoch);
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i)
        for (unsigned y = 0; y < HEIGHT; ++y)
            for (unsigned x = 0; x < WIDTH; ++x)
                cpu[i][y * STRIDE + x] = published[i][y * STRIDE + x] = 0x1234;
    p4pad_damage_invalidate_all(&damage);
    const int stale_index = p4dp_reserve(&owner);
    assert(stale_index == 1);
    assert(p4pad_damage_take(&damage, (unsigned)stale_index, &rect));
    expect_rect(rect, 0, 0, WIDTH, HEIGHT);
    assert(p4pad_damage_copy_rgb565(&damage, cpu[stale_index], STRIDE * HEIGHT, STRIDE,
                                  source, STRIDE * HEIGHT, STRIDE, &rect, rotate));
    if (reached_ready) assert(p4dp_ready(&owner, (uint8_t)stale_index));
    assert(p4dp_discard_prepared(&owner, (uint8_t)stale_index));
    assert(owner.scanning == 0 && owner.pending == -1);
    assert(!p4pad_damage_peek(&damage, (unsigned)stale_index, &rect));
    // CPU completion alone did not publish the replacement. Clearing only a
    // "dirty" flag cannot rebuild the consumed debt on the same free index.
    const int retry = p4dp_reserve(&owner);
    assert(retry == stale_index);
    assert(!p4pad_damage_take(&damage, (unsigned)retry, &rect));
    assert(published[retry][0] == 0x1234);
    assert(p4dp_discard_prepared(&owner, (uint8_t)retry));

    // Cancel must invalidate before retry. No source writes follow: restoring
    // every framebuffer must still recover the complete canonical Pad image.
    p4pad_damage_invalidate_all(&damage);
    for (unsigned i = 0; i < P4PAD_DAMAGE_BUFFER_COUNT; ++i) {
        assert(p4pad_damage_take(&damage, i, &rect));
        assert(p4pad_damage_copy_rgb565(&damage, cpu[i], STRIDE * HEIGHT, STRIDE,
                                      source, STRIDE * HEIGHT, STRIDE, &rect, rotate));
        p4pad_rect_t output;
        assert(p4pad_damage_map_rect(&damage, &rect, rotate, &output));
        // Model the DPI driver's full-width cache writeback of the touched
        // rows, independently of the copy's pixel loop and damage bookkeeping.
        for (int32_t y = output.y1; y < output.y2; ++y)
            memcpy(published[i] + y * STRIDE, cpu[i] + y * STRIDE, WIDTH * sizeof(uint16_t));
        independent_full_compare(published[i], STRIDE, source, STRIDE, WIDTH, HEIGHT, rotate);
        check_padding(cpu[i], WIDTH, HEIGHT, STRIDE, HEIGHT, 0xcafe);
        check_padding(published[i], WIDTH, HEIGHT, STRIDE, HEIGHT, 0xcafe);
    }
}

int main(void)
{
    debt_is_independent_clipped_and_unioned();
    rotation_and_row_spans_are_exact();
    distant_changes_do_not_copy_the_space_between_them();
    for (unsigned rotate = 0; rotate < 2; ++rotate) {
        small_copy_preserves_every_untouched_pixel(rotate != 0);
        three_buffers_catch_up_to_all_source_updates(17, 11, rotate != 0);
        three_buffers_catch_up_to_all_source_updates(1024, 600, rotate != 0);
        stale_job_cancellation_restores_unpublished_debt(rotate != 0, false);
        stale_job_cancellation_restores_unpublished_debt(rotate != 0, true);
    }
    puts("Pad per-buffer damage, rotation, bounds and independent pixel reference tests passed");
    return 0;
}
