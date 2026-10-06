#define P4DESK_CACHE_SYNC_HOST_TEST 1
#include "display_cache_sync.h"

#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#define IRAM_ATTR
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 0x102
#define ESP_ERR_INVALID_STATE 0x103
#define ESP_ERR_NO_MEM 0x101
#define ESP_ERR_NOT_SUPPORTED 0x106
#define ESP_CACHE_MSYNC_FLAG_INVALIDATE (1 << 0)
#define ESP_CACHE_MSYNC_FLAG_UNALIGNED (1 << 1)
#define ESP_CACHE_MSYNC_FLAG_DIR_C2M (1 << 2)
#define ESP_CACHE_MSYNC_FLAG_DIR_M2C (1 << 3)
#define ESP_CACHE_MSYNC_FLAG_TYPE_DATA (1 << 4)
#define ESP_CACHE_MSYNC_FLAG_TYPE_INST (1 << 5)
typedef void *TaskHandle_t;
typedef enum { CACHE_TYPE_DATA, CACHE_TYPE_INSTRUCTION } cache_type_t;

static bool in_isr;
static TaskHandle_t task = (void *)1;
static size_t line_size = 128;
static bool mapping_valid = true;
static int64_t clock_us;
static int64_t real_elapsed_us = 25;
static size_t timer_calls;
static size_t real_calls;
static size_t fail_call;
static const esp_err_t injected_error = 0x777;
typedef struct { uintptr_t addr; size_t bytes; int flags; } call_t;
static call_t calls[64];

static bool xPortInIsrContext(void) { return in_isr; }
static TaskHandle_t xTaskGetCurrentTaskHandle(void) { return task; }
static int64_t esp_timer_get_time(void) { ++timer_calls; return clock_us; }
static bool cache_hal_vaddr_to_cache_level_id(uint32_t base, uint32_t bytes,
                                             uint32_t *level, uint32_t *id)
{
    if (!mapping_valid || base < 0x48000000u || (uint64_t)base + bytes > 0x4c000000u) return false;
    *level = 2; *id = 0;
    return true;
}
static uint32_t cache_hal_get_cache_line_size(uint32_t level, cache_type_t type)
{
    (void)level; (void)type;
    return (uint32_t)line_size;
}

esp_err_t __real_esp_cache_msync(void *addr, size_t bytes, int flags)
{
    assert(real_calls < sizeof(calls) / sizeof(calls[0]));
    calls[real_calls++] = (call_t){(uintptr_t)addr, bytes, flags};
    clock_us += real_elapsed_us;
    if (fail_call == real_calls) return injected_error;
    // Mirror the fixed SDK validation, including zero-size forwarding. These
    // tests record calls rather than reading or invalidating synthetic memory.
    if (!addr || bytes > UINT32_MAX || (uint64_t)(uint32_t)(uintptr_t)addr + bytes > UINT32_MAX)
        return ESP_ERR_INVALID_ARG;
    if (((flags & ESP_CACHE_MSYNC_FLAG_DIR_C2M) && (flags & ESP_CACHE_MSYNC_FLAG_DIR_M2C)) ||
        ((flags & ESP_CACHE_MSYNC_FLAG_TYPE_DATA) && (flags & ESP_CACHE_MSYNC_FLAG_TYPE_INST)))
        return ESP_ERR_INVALID_ARG;
    uint32_t level, id;
    if (!cache_hal_vaddr_to_cache_level_id((uint32_t)(uintptr_t)addr, (uint32_t)bytes, &level, &id))
        return ESP_ERR_NOT_SUPPORTED;
    if (!(flags & ESP_CACHE_MSYNC_FLAG_UNALIGNED) &&
        ((uintptr_t)addr % line_size || bytes % line_size)) return ESP_ERR_INVALID_ARG;
    if ((flags & ESP_CACHE_MSYNC_FLAG_DIR_M2C) && (flags & ESP_CACHE_MSYNC_FLAG_UNALIGNED))
        return ESP_ERR_INVALID_ARG;
    if (!(flags & ESP_CACHE_MSYNC_FLAG_DIR_M2C) && (flags & ESP_CACHE_MSYNC_FLAG_TYPE_INST))
        return ESP_ERR_INVALID_ARG;
    return ESP_OK;
}

// Compile and execute the production implementation with only SDK entry
// points replaced. No duplicated wrapper algorithm is used by these tests.
#include "../../firmware/main/display_cache_sync.c"

static const uintptr_t lcd0 = 0x48000000u, lcd1 = 0x48200000u;
static const uintptr_t lcd2 = 0x48400000u, scratch = 0x48600000u;
static const size_t capacity = 1024u * 608u * 2u;
static const size_t visible = 1024u * 600u * 2u;

static void clear_calls(void)
{
    memset(calls, 0, sizeof(calls));
    real_calls = timer_calls = fail_call = 0;
}

static void expect_forward(uintptr_t addr, size_t bytes, int flags, esp_err_t expected)
{
    clear_calls();
    p4desk_cache_sync_stats_t before, after;
    p4desk_cache_sync_stats(&before);
    assert(__wrap_esp_cache_msync((void *)addr, bytes, flags) == expected);
    assert(real_calls == 1 && timer_calls == 0);
    assert(calls[0].addr == addr && calls[0].bytes == bytes && calls[0].flags == flags);
    p4desk_cache_sync_stats(&after);
    assert(memcmp(&before, &after, sizeof(before)) == 0);
}

static void test_registration(void)
{
    assert(p4desk_cache_sync_seal_outputs() == ESP_ERR_INVALID_STATE);
    assert(p4desk_cache_sync_register_output(NULL, capacity) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)lcd0, 0) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)(lcd0 + 1), capacity) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity - 1) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)(UINTPTR_MAX - 127), 256) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)0xffffff80u, 256) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)lcd0, (size_t)UINT32_MAX + 1) == ESP_ERR_INVALID_ARG);
    mapping_valid = false;
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_ERR_INVALID_ARG);
    mapping_valid = true;
    line_size = 0;
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_ERR_INVALID_ARG);
    line_size = 192;
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_ERR_INVALID_ARG);
    line_size = 65536;
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_ERR_INVALID_ARG);
    line_size = 128;
    in_isr = true;
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_ERR_INVALID_STATE);
    assert(p4desk_cache_sync_seal_outputs() == ESP_ERR_INVALID_STATE);
    in_isr = false;

    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_OK);
    assert(p4desk_cache_sync_register_output((void *)lcd0, capacity) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)(lcd0 + 128), 128) == ESP_ERR_INVALID_ARG);
    assert(p4desk_cache_sync_register_output((void *)(lcd0 - 128), 256) == ESP_ERR_INVALID_ARG);
    task = (void *)2;
    assert(p4desk_cache_sync_register_output((void *)lcd1, capacity) == ESP_ERR_INVALID_STATE);
    assert(p4desk_cache_sync_seal_outputs() == ESP_ERR_INVALID_STATE);
    task = (void *)1;
    assert(p4desk_cache_sync_register_output((void *)lcd1, capacity) == ESP_OK);
    assert(p4desk_cache_sync_register_output((void *)lcd2, capacity) == ESP_OK);
    assert(p4desk_cache_sync_register_output((void *)scratch, capacity) == ESP_OK);
    assert(p4desk_cache_sync_register_output((void *)0x48800000u, 128) == ESP_ERR_NO_MEM);
    expect_forward(lcd0, visible, ESP_CACHE_MSYNC_FLAG_DIR_M2C, ESP_OK);
    assert(p4desk_cache_sync_seal_outputs() == ESP_OK);
    assert(p4desk_cache_sync_seal_outputs() == ESP_ERR_INVALID_STATE);
    assert(p4desk_cache_sync_register_output((void *)0x48800000u, 128) == ESP_ERR_INVALID_STATE);
}

static void test_exact_chunk_coverage(void)
{
    const uintptr_t bases[] = {lcd0, lcd1, lcd2, scratch};
    const int flags = ESP_CACHE_MSYNC_FLAG_DIR_M2C | ESP_CACHE_MSYNC_FLAG_TYPE_DATA |
        ESP_CACHE_MSYNC_FLAG_INVALIDATE | (1 << 20); // Unknown SDK flag is preserved.
    for (size_t b = 0; b < sizeof(bases) / sizeof(bases[0]); ++b) {
        clear_calls();
        const size_t bytes = b == 3 ? capacity : visible;
        p4desk_cache_sync_stats_t before, after;
        p4desk_cache_sync_stats(&before);
        assert(__wrap_esp_cache_msync((void *)bases[b], bytes, flags) == ESP_OK);
        size_t covered = 0;
        for (size_t i = 0; i < real_calls; ++i) {
            assert(calls[i].addr == bases[b] + covered);
            assert(calls[i].bytes && calls[i].bytes <= P4DESK_CACHE_SYNC_CHUNK_BYTES);
            assert(calls[i].addr % 128 == 0 && calls[i].bytes % 128 == 0);
            assert(calls[i].flags == flags);
            covered += calls[i].bytes;
        }
        assert(covered == bytes && real_calls == 38 && timer_calls == real_calls * 2);
        assert(calls[37].bytes == (b == 3 ? 32768u : 16384u));
        p4desk_cache_sync_stats(&after);
        assert(after.whole_calls == before.whole_calls + 1);
        assert(after.chunks == before.chunks + real_calls && after.errors == before.errors);
        assert(after.max_chunk_us == 25);
    }
    clear_calls();
    assert(__wrap_esp_cache_msync((void *)(lcd0 + 128), 128, ESP_CACHE_MSYNC_FLAG_DIR_M2C) == ESP_OK);
    assert(real_calls == 1 && calls[0].addr == lcd0 + 128 && calls[0].bytes == 128);
}

static void test_original_forwarding(void)
{
    const int m2c = ESP_CACHE_MSYNC_FLAG_DIR_M2C;
    expect_forward(lcd0, visible, 0, ESP_OK); // Default C2M.
    expect_forward(lcd0, visible, ESP_CACHE_MSYNC_FLAG_DIR_C2M, ESP_OK);
    expect_forward(lcd0, visible, m2c | ESP_CACHE_MSYNC_FLAG_DIR_C2M, ESP_ERR_INVALID_ARG);
    expect_forward(lcd0, visible, m2c | ESP_CACHE_MSYNC_FLAG_TYPE_INST, ESP_OK);
    expect_forward(lcd0, visible, m2c | ESP_CACHE_MSYNC_FLAG_TYPE_INST | ESP_CACHE_MSYNC_FLAG_TYPE_DATA, ESP_ERR_INVALID_ARG);
    expect_forward(lcd0, visible, m2c | ESP_CACHE_MSYNC_FLAG_UNALIGNED, ESP_ERR_INVALID_ARG);
    expect_forward(lcd0 + 1, visible, m2c, ESP_ERR_INVALID_ARG);
    expect_forward(lcd0, visible - 1, m2c, ESP_ERR_INVALID_ARG);
    expect_forward(lcd0, 0, m2c, ESP_OK);
    expect_forward(0, visible, m2c, ESP_ERR_INVALID_ARG);
    expect_forward(UINTPTR_MAX - 127, 256, m2c, ESP_ERR_INVALID_ARG);
    expect_forward(0xffffff80u, 256, m2c, ESP_ERR_INVALID_ARG);
    expect_forward(lcd0, (size_t)UINT32_MAX + 1, m2c, ESP_ERR_INVALID_ARG);
    expect_forward(0x48800000u, visible, m2c, ESP_OK); // Undeclared allocation.
    expect_forward(lcd0 + capacity - 128, 256, m2c, ESP_OK); // Exceeds registered span.
    expect_forward(lcd0, lcd1 - lcd0 + 128, m2c, ESP_OK); // Spans two ranges.
    mapping_valid = false;
    expect_forward(lcd0, visible, m2c, ESP_ERR_NOT_SUPPORTED); // Whole span rejected before prefix work.
    mapping_valid = true;
    line_size = 256;
    expect_forward(lcd0, visible, m2c, ESP_OK); // Cached geometry changed since registration.
    line_size = 128;
    in_isr = true;
    expect_forward(lcd0, visible, m2c, ESP_OK);
    in_isr = false;
}

static void test_first_error_and_statistics(void)
{
    clear_calls();
    fail_call = 2;
    real_elapsed_us = 71;
    p4desk_cache_sync_stats_t before, after;
    p4desk_cache_sync_stats(&before);
    assert(__wrap_esp_cache_msync((void *)lcd0, visible, ESP_CACHE_MSYNC_FLAG_DIR_M2C) == injected_error);
    assert(real_calls == 2 && timer_calls == 4);
    assert(calls[0].addr == lcd0 && calls[1].addr == lcd0 + 32768);
    assert(calls[0].bytes == 32768 && calls[1].bytes == 32768);
    p4desk_cache_sync_stats(&after);
    assert(after.whole_calls == before.whole_calls + 1);
    assert(after.chunks == before.chunks + 2 && after.errors == before.errors + 1);
    assert(after.max_chunk_us == 71);
    p4desk_cache_sync_stats(NULL);
    atomic_store(&s_whole_calls, UINT32_MAX);
    atomic_store(&s_chunks, UINT32_MAX);
    atomic_store(&s_errors, UINT32_MAX);
    clear_calls();
    fail_call = 1;
    real_elapsed_us = (int64_t)UINT32_MAX + 17;
    assert(__wrap_esp_cache_msync((void *)lcd0, 128, ESP_CACHE_MSYNC_FLAG_DIR_M2C) == injected_error);
    p4desk_cache_sync_stats(&after);
    assert(after.whole_calls == UINT32_MAX && after.chunks == UINT32_MAX && after.errors == UINT32_MAX);
    assert(after.max_chunk_us == UINT32_MAX);
}

int main(void)
{
    test_registration();
    test_exact_chunk_coverage();
    test_original_forwarding();
    test_first_error_and_statistics();
    puts("display cache sync tests passed (ASan/UBSan)");
    return 0;
}
