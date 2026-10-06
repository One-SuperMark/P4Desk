#include "display_cache_sync.h"

#include <stdbool.h>
#include <stdatomic.h>

#ifndef P4DESK_CACHE_SYNC_HOST_TEST
#include "esp_attr.h"
#include "esp_cache.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "hal/cache_hal.h"
#endif

_Static_assert(__atomic_always_lock_free(sizeof(uint32_t), 0), "cache statistics must be lock-free");
_Static_assert(sizeof(atomic_uint_least32_t) == sizeof(uint32_t), "cache statistics must use 32-bit storage");
_Static_assert(ATOMIC_BOOL_LOCK_FREE == 2, "sealed range publication must be lock-free");
_Static_assert(ATOMIC_POINTER_LOCK_FREE == 2, "boot task claim must be lock-free");

typedef struct {
    uintptr_t base;
    uintptr_t end;
    size_t line_size;
} cache_output_t;

// Ordinary static BSS is internal RAM; no allocation or PSRAM context is used.
static cache_output_t s_outputs[P4DESK_CACHE_SYNC_OUTPUT_COUNT];
static size_t s_output_count;
static _Atomic(TaskHandle_t) s_registration_task;
static atomic_bool s_sealed;
static atomic_uint_least32_t s_whole_calls, s_chunks, s_errors, s_max_chunk_us;

extern esp_err_t __real_esp_cache_msync(void *addr, size_t size, int flags);

static bool claim_boot_task(void)
{
    const TaskHandle_t caller = xTaskGetCurrentTaskHandle();
    TaskHandle_t expected = NULL;
    return caller &&
        (atomic_compare_exchange_strong_explicit(&s_registration_task, &expected, caller,
                                                 memory_order_relaxed, memory_order_relaxed) ||
         expected == caller);
}

static bool mapped_data_span(uintptr_t base, size_t bytes, size_t *line_size)
{
    uintptr_t end;
    if (!base || !bytes || bytes > UINT32_MAX || base > UINT32_MAX ||
        __builtin_add_overflow(base, bytes, &end) || end > UINT32_MAX) return false;
    uint32_t level = 0, id = 0;
    if (!cache_hal_vaddr_to_cache_level_id((uint32_t)base, (uint32_t)bytes, &level, &id)) return false;
    const size_t line = cache_hal_get_cache_line_size(level, CACHE_TYPE_DATA);
    if (!line || (line & (line - 1)) || line > P4DESK_CACHE_SYNC_CHUNK_BYTES ||
        P4DESK_CACHE_SYNC_CHUNK_BYTES % line) return false;
    *line_size = line;
    return true;
}

esp_err_t p4desk_cache_sync_register_output(void *base, size_t capacity)
{
    if (xPortInIsrContext() || atomic_load_explicit(&s_sealed, memory_order_acquire))
        return ESP_ERR_INVALID_STATE;
    size_t line = 0;
    const uintptr_t start = (uintptr_t)base;
    if (!mapped_data_span(start, capacity, &line) || start % line || capacity % line)
        return ESP_ERR_INVALID_ARG;
    if (!claim_boot_task()) return ESP_ERR_INVALID_STATE;
    const uintptr_t end = start + capacity; // Validated by mapped_data_span.
    for (size_t i = 0; i < s_output_count; ++i) {
        if (start < s_outputs[i].end && s_outputs[i].base < end) return ESP_ERR_INVALID_ARG;
    }
    if (s_output_count == P4DESK_CACHE_SYNC_OUTPUT_COUNT) return ESP_ERR_NO_MEM;
    s_outputs[s_output_count++] = (cache_output_t){start, end, line};
    return ESP_OK;
}

esp_err_t p4desk_cache_sync_seal_outputs(void)
{
    if (xPortInIsrContext() || atomic_load_explicit(&s_sealed, memory_order_acquire) ||
        !claim_boot_task() || !s_output_count) return ESP_ERR_INVALID_STATE;
    // A wrapper seeing true also sees every preceding immutable range write.
    atomic_store_explicit(&s_sealed, true, memory_order_release);
    return ESP_OK;
}

static void saturating_increment(atomic_uint_least32_t *counter)
{
    uint_least32_t value = atomic_load_explicit(counter, memory_order_relaxed);
    while (value != UINT32_MAX &&
           !atomic_compare_exchange_weak_explicit(counter, &value, value + 1,
                                                 memory_order_relaxed, memory_order_relaxed)) {}
}

static void record_chunk_duration(int64_t elapsed)
{
    const uint_least32_t duration = elapsed <= 0 ? 0 :
        elapsed > UINT32_MAX ? UINT32_MAX : (uint32_t)elapsed;
    uint_least32_t previous = atomic_load_explicit(&s_max_chunk_us, memory_order_relaxed);
    while (duration > previous &&
           !atomic_compare_exchange_weak_explicit(&s_max_chunk_us, &previous, duration,
                                                 memory_order_relaxed, memory_order_relaxed)) {}
}

void p4desk_cache_sync_stats(p4desk_cache_sync_stats_t *out)
{
    if (!out) return;
    *out = (p4desk_cache_sync_stats_t){
        atomic_load_explicit(&s_whole_calls, memory_order_relaxed),
        atomic_load_explicit(&s_chunks, memory_order_relaxed),
        atomic_load_explicit(&s_errors, memory_order_relaxed),
        atomic_load_explicit(&s_max_chunk_us, memory_order_relaxed),
    };
}

static bool eligible_data_m2c(uintptr_t base, size_t bytes, int flags)
{
    // Preserve the SDK's validation and defaults by forwarding invalid flags,
    // unaligned requests (even with aligned addresses), and instruction sync.
    if (!(flags & ESP_CACHE_MSYNC_FLAG_DIR_M2C) ||
        (flags & (ESP_CACHE_MSYNC_FLAG_DIR_C2M | ESP_CACHE_MSYNC_FLAG_UNALIGNED |
                  ESP_CACHE_MSYNC_FLAG_TYPE_INST))) return false;
    size_t line = 0;
    // Whole-span HAL validation occurs before touching a prefix of the range.
    if (!mapped_data_span(base, bytes, &line) || base % line || bytes % line) return false;
    const uintptr_t end = base + bytes;
    for (size_t i = 0; i < s_output_count; ++i) {
        if (base >= s_outputs[i].base && end <= s_outputs[i].end &&
            line == s_outputs[i].line_size) return true;
    }
    return false;
}

esp_err_t IRAM_ATTR __wrap_esp_cache_msync(void *addr, size_t size, int flags)
{
    // The wrapper can be reached from SDK ISRs. Take this branch before any
    // flash-resident helper, HAL validation, timing, or statistics work.
    if (xPortInIsrContext()) return __real_esp_cache_msync(addr, size, flags);
    if (!atomic_load_explicit(&s_sealed, memory_order_acquire) ||
        !eligible_data_m2c((uintptr_t)addr, size, flags))
        return __real_esp_cache_msync(addr, size, flags);

    saturating_increment(&s_whole_calls);
    size_t offset = 0;
    while (offset < size) {
        const size_t remaining = size - offset;
        const size_t chunk = remaining > P4DESK_CACHE_SYNC_CHUNK_BYTES ?
            P4DESK_CACHE_SYNC_CHUNK_BYTES : remaining;
        saturating_increment(&s_chunks);
        const int64_t before = esp_timer_get_time();
        const esp_err_t result = __real_esp_cache_msync((void *)((uintptr_t)addr + offset), chunk, flags);
        record_chunk_duration(esp_timer_get_time() - before);
        if (result != ESP_OK) {
            saturating_increment(&s_errors);
            return result;
        }
        offset += chunk;
    }
    return ESP_OK;
}
