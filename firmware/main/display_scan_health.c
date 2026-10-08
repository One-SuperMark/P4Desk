/* SPDX-License-Identifier: MIT */
#include "display_scan_health.h"

#include <limits.h>
#include <string.h>

void p4dsh_init(p4dsh_state_t *state)
{
    if (state) memset(state, 0, sizeof(*state));
}

/* A decrease rebases a replaced/reset counter instead of inventing a fault. */
static bool newer_fault(uint32_t count, uint32_t previous_count,
                        int64_t at_us, int64_t previous_us,
                        bool latest_is_fault)
{
    return count > previous_count ||
           (count == UINT32_MAX && previous_count == UINT32_MAX &&
            at_us >= 0 && at_us > previous_us && latest_is_fault);
}

p4dsh_result_t p4dsh_poll(p4dsh_state_t *state,
                         const p4dsh_snapshot_t *snapshot,
                         int64_t now_us)
{
    p4dsh_result_t result = { .action = P4DSH_IDLE, .reasons = 0 };
    if (!state || !snapshot) return result;

    const p4dsh_snapshot_t *previous = &state->previous;
    if (newer_fault(snapshot->bridge_underruns, previous->bridge_underruns,
                    snapshot->bridge_last_us, previous->bridge_last_us, true))
        state->pending_reasons |= P4DSH_REASON_BRIDGE;
    if (newer_fault(snapshot->host_overflows, previous->host_overflows,
                    snapshot->host_last_us, previous->host_last_us,
                    (snapshot->host_last_status1 & P4DSH_HOST_DPI_OVERFLOW) != 0))
        state->pending_reasons |= P4DSH_REASON_HOST_OVERFLOW;
    if (newer_fault(snapshot->host_underflows, previous->host_underflows,
                    snapshot->host_last_us, previous->host_last_us,
                    (snapshot->host_last_status1 & P4DSH_HOST_DPI_UNDERFLOW) != 0))
        state->pending_reasons |= P4DSH_REASON_HOST_UNDERFLOW;
    state->previous = *snapshot;
    result.reasons = state->pending_reasons;

    if (now_us < 0 || (state->has_clock && now_us < state->last_poll_us)) {
        if (result.reasons) result.action = P4DSH_INVALID_CLOCK;
        return result;
    }
    state->has_clock = true;
    state->last_poll_us = now_us;
    if (!result.reasons) return result;

    if (state->attempt_count) {
        unsigned latest = (state->attempt_head + state->attempt_count - 1) %
                          P4DSH_MAX_ATTEMPTS;
        /* Both operands are nonnegative and ordered, so subtraction is safe. */
        if (now_us - state->attempts_us[latest] < P4DSH_COOLDOWN_US) {
            result.action = P4DSH_COOLDOWN;
            return result;
        }
    }
    /* Three entries are a strict bound, independent of poll/event frequency. */
    while (state->attempt_count &&
           now_us - state->attempts_us[state->attempt_head] >= P4DSH_WINDOW_US) {
        state->attempt_head = (state->attempt_head + 1) % P4DSH_MAX_ATTEMPTS;
        --state->attempt_count;
    }
    if (state->attempt_count == P4DSH_MAX_ATTEMPTS) {
        result.action = P4DSH_THROTTLED;
        return result;
    }

    unsigned next = (state->attempt_head + state->attempt_count) %
                    P4DSH_MAX_ATTEMPTS;
    state->attempts_us[next] = now_us;
    ++state->attempt_count;
    state->pending_reasons = 0;
    result.action = P4DSH_RECOVER;
    return result;
}
