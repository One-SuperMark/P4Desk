#include "time_sync_state.h"
#include <assert.h>
#include <stdio.h>

static void auto_sync_and_hourly_refresh(void) {
    p4_time_sync_state_t s = {0};
    assert(p4_time_sync_step(&s, false, false, 0) == P4_TIME_NONE);
    assert(p4_time_sync_step(&s, true, false, 2000) == P4_TIME_START);
    assert(p4_time_sync_accepts(&s, 2200));
    p4_time_sync_success(&s, 1790748000U, 2200);
    assert(!p4_time_sync_accepts(&s, 2200));
    assert(p4_time_sync_step(&s, true, false, 2300) == P4_TIME_STOP);
    assert(s.status.success_count == 1 && s.status.phase == P4_TIME_SYNCED);
    assert(p4_time_sync_step(&s, true, false, 3602199) == P4_TIME_NONE);
    assert(p4_time_sync_step(&s, true, false, 3602200) == P4_TIME_START);
}
static void timeout_retry_and_late_response(void) {
    p4_time_sync_state_t s = {0};
    assert(p4_time_sync_step(&s, true, false, 0) == P4_TIME_START);
    // Tapping during a pending DNS/UDP operation neither extends it nor starts another.
    assert(p4_time_sync_step(&s, true, true, 59000) == P4_TIME_NONE);
    assert(!p4_time_sync_accepts(&s, 60000));
    assert(p4_time_sync_step(&s, true, false, 60000) == P4_TIME_STOP);
    assert(s.status.error == P4_TIME_TIMEOUT && s.status.success_count == 0);
    assert(!p4_time_sync_accepts(&s, 60001));
    assert(p4_time_sync_step(&s, true, false, 359999) == P4_TIME_NONE);
    assert(p4_time_sync_step(&s, true, false, 360000) == P4_TIME_START);
}
static void disconnect_and_reconnect_preserve_last_success(void) {
    p4_time_sync_state_t s = {0};
    p4_time_sync_step(&s, true, false, 0);
    p4_time_sync_success(&s, 1790748000U, 500);
    assert(p4_time_sync_step(&s, false, true, 600) == P4_TIME_STOP);
    assert(s.status.phase == P4_TIME_OFFLINE && s.status.last_sync_unix_s == 1790748000U);
    assert(!p4_time_sync_accepts(&s, 700));
    assert(p4_time_sync_step(&s, false, true, 800) == P4_TIME_NONE);
    assert(p4_time_sync_step(&s, true, false, 900) == P4_TIME_START);
    assert(p4_time_sync_step(&s, false, false, 950) == P4_TIME_STOP);
}
static void manual_retry_and_failures_preserve_clock_record(void) {
    p4_time_sync_state_t s = {0};
    p4_time_sync_step(&s, true, false, 0);
    p4_time_sync_success(&s, 1790748000U, 50);
    p4_time_sync_step(&s, true, false, 60);
    for (uint32_t error = P4_TIME_TIMEOUT; error <= P4_TIME_APPLY; error++) {
        assert(p4_time_sync_step(&s, true, true, 100) == P4_TIME_START);
        p4_time_sync_failure(&s, error, 200);
        assert(p4_time_sync_step(&s, true, false, 210) == P4_TIME_STOP);
        assert(s.status.phase == P4_TIME_RETRY && s.status.error == error);
        assert(s.status.last_sync_unix_s == 1790748000U && s.status.success_count == 1);
    }
}
static void invalid_epochs_and_post_2038_time(void) {
    assert(!p4_time_sync_valid(-1, 0));
    assert(!p4_time_sync_valid(946684799LL, 999999));
    assert(p4_time_sync_valid(946684800LL, 0));
    assert(p4_time_sync_valid(2147483648LL, 999999));
    assert(p4_time_sync_valid(4102444800LL, 0));
    assert(!p4_time_sync_valid(4102444801LL, 0));
    assert(!p4_time_sync_valid(INT64_MAX, 0));
    assert(!p4_time_sync_valid(1790748000LL, -1));
    assert(!p4_time_sync_valid(1790748000LL, 1000000));
    assert(!p4_time_sync_valid(4102444800LL, 1000));
}
int main(void) {
    auto_sync_and_hourly_refresh();
    timeout_retry_and_late_response();
    disconnect_and_reconnect_preserve_last_success();
    manual_retry_and_failures_preserve_clock_record();
    invalid_epochs_and_post_2038_time();
    puts("time sync: 5 scenarios passed (ASan/UBSan)");
}
