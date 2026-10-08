/* SPDX-License-Identifier: MIT */
#include "display_scan_health.h"

#include <assert.h>
#include <limits.h>
#include <stdio.h>
#include <string.h>

static p4dsh_result_t poll(p4dsh_state_t *state, p4dsh_snapshot_t *snapshot,
                           int64_t now_us, p4dsh_action_t expected)
{
    p4dsh_result_t result = p4dsh_poll(state, snapshot, now_us);
    assert(result.action == expected);
    assert(state->attempt_count <= P4DSH_MAX_ATTEMPTS);
    assert(state->attempt_head < P4DSH_MAX_ATTEMPTS);
    return result;
}

static void empty_and_unrelated_host_errors_do_nothing(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {0};
    p4dsh_init(&state);
    assert(poll(&state, &snapshot, 0, P4DSH_IDLE).reasons == 0);
    /* Startup status0 bit 20 is absent by design; other status1 bits ignored. */
    snapshot.host_last_us = 1;
    snapshot.host_last_status1 = UINT32_C(1) << 20;
    assert(poll(&state, &snapshot, 1, P4DSH_IDLE).reasons == 0);
    snapshot.host_last_us = INT64_MAX;
    snapshot.host_last_status1 = ~(P4DSH_HOST_DPI_OVERFLOW | P4DSH_HOST_DPI_UNDERFLOW);
    assert(poll(&state, &snapshot, INT64_MAX, P4DSH_IDLE).reasons == 0);
    assert(state.attempt_count == 0);
}

static void first_snapshot_does_not_swallow_true_startup_faults(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {
        .bridge_underruns = 4, .bridge_last_us = 0,
        .host_overflows = 2, .host_underflows = 3,
    };
    p4dsh_init(&state);
    p4dsh_result_t result = poll(&state, &snapshot, 0, P4DSH_RECOVER);
    assert(result.reasons == (P4DSH_REASON_BRIDGE | P4DSH_REASON_HOST_OVERFLOW |
                              P4DSH_REASON_HOST_UNDERFLOW));
    assert(state.attempt_count == 1);
    for (int64_t now = 1; now < 100; ++now)
        assert(poll(&state, &snapshot, now, P4DSH_IDLE).reasons == 0);
    poll(&state, &snapshot, P4DSH_WINDOW_US * 10, P4DSH_IDLE);
    assert(state.attempt_count == 1); /* No repeated recovery just for an old fault. */
}

static void cooldown_coalesces_a_burst_and_keeps_it_pending(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {.bridge_underruns = 1};
    p4dsh_init(&state);
    poll(&state, &snapshot, 10, P4DSH_RECOVER);
    snapshot.bridge_underruns = 100;
    snapshot.bridge_last_us = 11;
    assert(poll(&state, &snapshot, 11, P4DSH_COOLDOWN).reasons == P4DSH_REASON_BRIDGE);
    snapshot.host_underflows = 1;
    snapshot.host_last_us = 12;
    snapshot.host_last_status1 = P4DSH_HOST_DPI_UNDERFLOW;
    const uint32_t reasons = P4DSH_REASON_BRIDGE | P4DSH_REASON_HOST_UNDERFLOW;
    assert(poll(&state, &snapshot, 12, P4DSH_COOLDOWN).reasons == reasons);
    assert(poll(&state, &snapshot, P4DSH_COOLDOWN_US + 9, P4DSH_COOLDOWN).reasons == reasons);
    assert(poll(&state, &snapshot, P4DSH_COOLDOWN_US + 10, P4DSH_RECOVER).reasons == reasons);
    assert(poll(&state, &snapshot, P4DSH_COOLDOWN_US + 11, P4DSH_IDLE).reasons == 0);
    assert(state.attempt_count == 2);
}

static void three_attempts_are_bounded_in_a_rolling_window(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {0};
    p4dsh_init(&state);
    for (uint32_t event = 1; event <= 3; ++event) {
        snapshot.bridge_underruns = event;
        poll(&state, &snapshot, (event - 1) * P4DSH_COOLDOWN_US, P4DSH_RECOVER);
    }
    snapshot.bridge_underruns = 4;
    poll(&state, &snapshot, 2 * P4DSH_COOLDOWN_US + 1, P4DSH_COOLDOWN);
    poll(&state, &snapshot, 3 * P4DSH_COOLDOWN_US, P4DSH_THROTTLED);
    for (int64_t now = 4 * P4DSH_COOLDOWN_US; now < P4DSH_WINDOW_US; now += 1000000)
        poll(&state, &snapshot, now, P4DSH_THROTTLED);
    poll(&state, &snapshot, P4DSH_WINDOW_US - 1, P4DSH_THROTTLED);
    poll(&state, &snapshot, P4DSH_WINDOW_US, P4DSH_RECOVER);
    snapshot.bridge_underruns = 5;
    poll(&state, &snapshot, P4DSH_WINDOW_US + P4DSH_COOLDOWN_US - 1, P4DSH_COOLDOWN);
    poll(&state, &snapshot, P4DSH_WINDOW_US + P4DSH_COOLDOWN_US, P4DSH_RECOVER);
    snapshot.bridge_underruns = 6;
    poll(&state, &snapshot, P4DSH_WINDOW_US + 2 * P4DSH_COOLDOWN_US, P4DSH_RECOVER);
    snapshot.bridge_underruns = 7;
    poll(&state, &snapshot, P4DSH_WINDOW_US + 3 * P4DSH_COOLDOWN_US, P4DSH_THROTTLED);
    poll(&state, &snapshot, 2 * P4DSH_WINDOW_US, P4DSH_RECOVER);
}

static void expired_attempts_are_removed_even_after_long_idle(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {.host_overflows = 1};
    p4dsh_init(&state);
    poll(&state, &snapshot, 0, P4DSH_RECOVER);
    snapshot.host_overflows = 2;
    poll(&state, &snapshot, P4DSH_COOLDOWN_US, P4DSH_RECOVER);
    snapshot.host_overflows = 3;
    poll(&state, &snapshot, 2 * P4DSH_COOLDOWN_US, P4DSH_RECOVER);
    snapshot.host_overflows = 4;
    poll(&state, &snapshot, 5 * P4DSH_WINDOW_US, P4DSH_RECOVER);
    assert(state.attempt_count == 1);
}

static void saturated_bridge_counter_uses_its_specific_timestamp(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {.bridge_underruns = UINT32_MAX, .bridge_last_us = 1};
    p4dsh_init(&state);
    poll(&state, &snapshot, 1, P4DSH_RECOVER);
    poll(&state, &snapshot, P4DSH_COOLDOWN_US + 1, P4DSH_IDLE);
    snapshot.bridge_last_us = 2;
    assert(poll(&state, &snapshot, P4DSH_COOLDOWN_US + 2, P4DSH_RECOVER).reasons == P4DSH_REASON_BRIDGE);
    snapshot.bridge_last_us = -1; /* Invalid/rewound timestamps cannot invent new faults. */
    poll(&state, &snapshot, 2 * P4DSH_COOLDOWN_US + 2, P4DSH_IDLE);
}

static void saturated_host_counter_requires_a_new_matching_fifo_observation(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {
        .host_overflows = UINT32_MAX, .host_underflows = UINT32_MAX,
        .host_last_us = 1,
        .host_last_status1 = P4DSH_HOST_DPI_OVERFLOW | P4DSH_HOST_DPI_UNDERFLOW,
    };
    p4dsh_init(&state);
    poll(&state, &snapshot, 1, P4DSH_RECOVER);
    snapshot.host_last_us = 2;
    snapshot.host_last_status1 = UINT32_C(1) << 20;
    poll(&state, &snapshot, P4DSH_COOLDOWN_US + 1, P4DSH_IDLE);
    snapshot.host_last_status1 = P4DSH_HOST_DPI_UNDERFLOW;
    poll(&state, &snapshot, P4DSH_COOLDOWN_US + 2, P4DSH_IDLE); /* Same observation. */
    snapshot.host_last_us = 3;
    assert(poll(&state, &snapshot, P4DSH_COOLDOWN_US + 3, P4DSH_RECOVER).reasons == P4DSH_REASON_HOST_UNDERFLOW);
    snapshot.host_last_us = 4;
    snapshot.host_last_status1 = P4DSH_HOST_DPI_OVERFLOW;
    assert(poll(&state, &snapshot, 2 * P4DSH_COOLDOWN_US + 3, P4DSH_RECOVER).reasons == P4DSH_REASON_HOST_OVERFLOW);
    snapshot.host_last_us = 2; /* A stale read is not a new observation. */
    poll(&state, &snapshot, 3 * P4DSH_COOLDOWN_US + 3, P4DSH_IDLE);
}

static void counter_decrease_rebases_without_triggering_a_false_fault(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {.bridge_underruns = 10, .host_underflows = 10};
    p4dsh_init(&state);
    poll(&state, &snapshot, 0, P4DSH_RECOVER);
    snapshot.bridge_underruns = 0;
    snapshot.host_underflows = 0;
    snapshot.bridge_last_us = 1;
    snapshot.host_last_us = 1;
    poll(&state, &snapshot, P4DSH_COOLDOWN_US, P4DSH_IDLE);
    snapshot.host_underflows = 1;
    assert(poll(&state, &snapshot, P4DSH_COOLDOWN_US + 1, P4DSH_RECOVER).reasons == P4DSH_REASON_HOST_UNDERFLOW);
}

static void invalid_or_backwards_clock_keeps_events_without_bypassing_limits(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {.bridge_underruns = 1};
    p4dsh_init(&state);
    poll(&state, &snapshot, INT64_MIN, P4DSH_INVALID_CLOCK);
    poll(&state, &snapshot, -1, P4DSH_INVALID_CLOCK);
    assert(state.attempt_count == 0);
    poll(&state, &snapshot, 100, P4DSH_RECOVER);
    poll(&state, &snapshot, 200, P4DSH_IDLE);
    snapshot.bridge_underruns = 2;
    poll(&state, &snapshot, 199, P4DSH_INVALID_CLOCK);
    poll(&state, &snapshot, 100, P4DSH_INVALID_CLOCK);
    poll(&state, &snapshot, 200, P4DSH_COOLDOWN);
    poll(&state, &snapshot, P4DSH_COOLDOWN_US + 100, P4DSH_RECOVER);
    snapshot.bridge_underruns = 3;
    poll(&state, &snapshot, INT64_MAX, P4DSH_RECOVER);
    snapshot.bridge_underruns = 4;
    poll(&state, &snapshot, INT64_MAX, P4DSH_COOLDOWN);
    poll(&state, &snapshot, INT64_MIN, P4DSH_INVALID_CLOCK);
}

static void null_arguments_are_safe_and_do_not_mutate(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {.bridge_underruns = 1};
    p4dsh_init(NULL);
    p4dsh_init(&state);
    p4dsh_state_t original = state;
    assert(p4dsh_poll(NULL, &snapshot, 1).action == P4DSH_IDLE);
    assert(p4dsh_poll(&state, NULL, 1).action == P4DSH_IDLE);
    assert(memcmp(&state, &original, sizeof(state)) == 0);
}

static void sustained_faults_and_irregular_polling_keep_the_rolling_budget(void)
{
    p4dsh_state_t state;
    p4dsh_snapshot_t snapshot = {0};
    p4dsh_init(&state);
    int64_t recent[P4DSH_MAX_ATTEMPTS] = {0};
    unsigned count = 0;
    int64_t now = 0;
    uint32_t random = UINT32_C(0x54123);
    for (unsigned i = 0; i < 50000; ++i) {
        random = random * UINT32_C(1664525) + UINT32_C(1013904223);
        now += (random % 2000000) + 1;
        if (random & 1) ++snapshot.host_underflows;
        p4dsh_result_t result = p4dsh_poll(&state, &snapshot, now);
        assert(state.attempt_count <= P4DSH_MAX_ATTEMPTS);
        if (result.action != P4DSH_RECOVER) continue;
        if (count) assert(now - recent[(count - 1) % P4DSH_MAX_ATTEMPTS] >= P4DSH_COOLDOWN_US);
        if (count >= P4DSH_MAX_ATTEMPTS)
            assert(now - recent[count % P4DSH_MAX_ATTEMPTS] >= P4DSH_WINDOW_US);
        recent[count % P4DSH_MAX_ATTEMPTS] = now;
        ++count;
    }
    assert(count > 100 && count < 3000);
}

int main(void)
{
    empty_and_unrelated_host_errors_do_nothing();
    first_snapshot_does_not_swallow_true_startup_faults();
    cooldown_coalesces_a_burst_and_keeps_it_pending();
    three_attempts_are_bounded_in_a_rolling_window();
    expired_attempts_are_removed_even_after_long_idle();
    saturated_bridge_counter_uses_its_specific_timestamp();
    saturated_host_counter_requires_a_new_matching_fifo_observation();
    counter_decrease_rebases_without_triggering_a_false_fault();
    invalid_or_backwards_clock_keeps_events_without_bypassing_limits();
    null_arguments_are_safe_and_do_not_mutate();
    sustained_faults_and_irregular_polling_keep_the_rolling_budget();
    puts("display scan health: 11 tests passed (ASan/UBSan)");
    return 0;
}
