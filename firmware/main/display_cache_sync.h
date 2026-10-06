#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef P4DESK_CACHE_SYNC_HOST_TEST
typedef int esp_err_t;
#else
#include "esp_err.h"
#endif

#define P4DESK_CACHE_SYNC_OUTPUT_COUNT 4u
#define P4DESK_CACHE_SYNC_CHUNK_BYTES (32u * 1024u)

typedef struct {
    uint32_t whole_calls; // Eligible registered M2C requests, not all SDK calls.
    uint32_t chunks;      // Actual __real_esp_cache_msync calls for those requests.
    uint32_t errors;      // Eligible requests stopped on their first SDK error.
    uint32_t max_chunk_us; // Elapsed real call time, including preemption; not IRQ-off duration.
} p4desk_cache_sync_stats_t;

/**
 * Register an entire cache-line-aligned DMA output allocation at boot.
 * Register/seal calls must come from one task, before display/DMA consumer
 * tasks start. Ranges cannot overlap, are never removed, and become immutable
 * after sealing. The caller remains responsible for exclusive buffer ownership.
 */
esp_err_t p4desk_cache_sync_register_output(void *base, size_t capacity);
esp_err_t p4desk_cache_sync_seal_outputs(void);

/** Lock-free numeric snapshot; fields can originate from adjacent instants. */
void p4desk_cache_sync_stats(p4desk_cache_sync_stats_t *out);

/**
 * Link using --wrap=esp_cache_msync. Only task-context, aligned data M2C inside
 * one sealed range is split; other calls, including ISR/C2M/invalid arguments,
 * are forwarded unchanged. Splitting replaces, rather than adds to, SDK sync.
 */
esp_err_t __wrap_esp_cache_msync(void *addr, size_t size, int flags);
