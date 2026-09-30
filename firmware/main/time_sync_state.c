#include "time_sync_state.h"

bool p4_time_sync_accepts(const p4_time_sync_state_t *s, int64_t now_ms) {
    return s->online && s->active && s->status.phase == P4_TIME_SYNCING && now_ms < s->deadline_ms;
}
bool p4_time_sync_valid(int64_t seconds, int32_t microseconds) {
    // Same UTC 2000-01-01 .. 2100-01-01 bounds as USB/manual time, checked before applying.
    return seconds >= 946684800LL && seconds <= 4102444800LL && microseconds >= 0 &&
           microseconds < 1000000 && (seconds < 4102444800LL || microseconds < 1000);
}
void p4_time_sync_success(p4_time_sync_state_t *s, uint32_t unix_s, int64_t now_ms) {
    s->status.phase = P4_TIME_SYNCED;
    s->status.error = P4_TIME_OK;
    s->status.last_sync_unix_s = unix_s;
    ++s->status.success_count;
    s->next_ms = now_ms + P4_TIME_INTERVAL_MS;
}
void p4_time_sync_failure(p4_time_sync_state_t *s, uint32_t error, int64_t now_ms) {
    s->status.phase = P4_TIME_RETRY;
    s->status.error = error;
    s->next_ms = now_ms + P4_TIME_RETRY_MS;
}
int p4_time_sync_step(p4_time_sync_state_t *s, bool online, bool request, int64_t now_ms) {
    if (!online) {
        bool stop = s->active;
        s->online = s->active = false;
        s->status.phase = P4_TIME_OFFLINE;
        s->status.error = P4_TIME_OK;
        return stop ? P4_TIME_STOP : P4_TIME_NONE;
    }
    if (!s->online) {
        s->online = true;
        s->next_ms = now_ms;
    }
    if (s->active) {
        // Repeated taps must not extend the timeout or create extra clients.
        if (s->status.phase == P4_TIME_SYNCING && now_ms >= s->deadline_ms)
            p4_time_sync_failure(s, P4_TIME_TIMEOUT, now_ms);
        if (s->status.phase != P4_TIME_SYNCING) {
            s->active = false;
            return P4_TIME_STOP;
        }
    } else if (request || now_ms >= s->next_ms) {
        s->active = true;
        s->deadline_ms = now_ms + P4_TIME_TIMEOUT_MS;
        s->status.phase = P4_TIME_SYNCING;
        s->status.error = P4_TIME_OK;
        return P4_TIME_START;
    }
    return P4_TIME_NONE;
}
