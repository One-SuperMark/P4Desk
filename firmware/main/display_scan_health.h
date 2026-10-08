/* SPDX-License-Identifier: MIT */
#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define P4DSH_COOLDOWN_US INT64_C(3000000)
#define P4DSH_WINDOW_US INT64_C(60000000)
#define P4DSH_MAX_ATTEMPTS 3
#define P4DSH_HOST_DPI_OVERFLOW (UINT32_C(1) << 7)
#define P4DSH_HOST_DPI_UNDERFLOW (UINT32_C(1) << 19)

/* Values copied from coherent diagnostic snapshots, never hardware registers. */
typedef struct {
    uint32_t bridge_underruns;
    int64_t bridge_last_us;
    uint32_t host_overflows;
    uint32_t host_underflows;
    int64_t host_last_us;
    uint32_t host_last_status1;
} p4dsh_snapshot_t;

enum {
    P4DSH_REASON_BRIDGE = 1u << 0,
    P4DSH_REASON_HOST_OVERFLOW = 1u << 1,
    P4DSH_REASON_HOST_UNDERFLOW = 1u << 2,
};

typedef enum {
    P4DSH_IDLE,
    P4DSH_RECOVER,
    P4DSH_COOLDOWN,
    P4DSH_THROTTLED,
    P4DSH_INVALID_CLOCK,
} p4dsh_action_t;

typedef struct {
    p4dsh_action_t action;
    uint32_t reasons;
} p4dsh_result_t;

typedef struct {
    p4dsh_snapshot_t previous;
    int64_t attempts_us[P4DSH_MAX_ATTEMPTS];
    int64_t last_poll_us;
    uint32_t pending_reasons;
    uint8_t attempt_head;
    uint8_t attempt_count;
    bool has_clock;
} p4dsh_state_t;

/* Zero baseline: the first snapshot must not hide an actual startup FIFO fault. */
void p4dsh_init(p4dsh_state_t *state);

/**
 * Observe new scan faults and reserve at most one recovery attempt.
 *
 * Call only from the display owner, after its read-clear Host poll and coherent
 * RAM snapshots. Counters are cumulative and saturating. Ordinary Host errors
 * (including startup status0 bit 20) are intentionally absent from this API.
 * At saturation, a new timestamp and the corresponding last Host FIFO flag
 * distinguish another fault from an unrelated Host error. Bridge timestamps
 * are specific to underruns. Repeated snapshots never create another request.
 *
 * RECOVER consumes an attempt even if the driver's subsequent recovery fails.
 * New errors observed during cooldown or rate limiting are coalesced and kept
 * until an attempt is allowed. There are at most 3 attempts in any rolling
 * 60-second window and at least 3 seconds between attempts; the first is
 * immediate. A backward/negative monotonic clock cannot bypass these limits.
 *
 * This policy neither reads registers nor changes framebuffer ownership. It
 * allocates nothing and has bounded O(1) work. Reinitialize it when replacing
 * the panel (its diagnostic counter lifetime); do not reinitialize on recovery
 * of the same panel. A NULL argument yields IDLE without changing state.
 */
p4dsh_result_t p4dsh_poll(p4dsh_state_t *state,
                         const p4dsh_snapshot_t *snapshot,
                         int64_t now_us);

#ifdef __cplusplus
}
#endif
