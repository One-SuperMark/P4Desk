#pragma once
#include <stdbool.h>
#include <stdint.h>

// Internal C/Rust snapshot ABI; never sent over USB.
typedef struct {
    uint32_t phase, error, last_sync_unix_s, success_count;
} p4_time_sync_snapshot_t;
_Static_assert(sizeof(p4_time_sync_snapshot_t) == 16, "time sync ABI");
enum { P4_TIME_OFFLINE, P4_TIME_SYNCING, P4_TIME_SYNCED, P4_TIME_RETRY };
enum { P4_TIME_OK, P4_TIME_TIMEOUT, P4_TIME_INIT, P4_TIME_INVALID, P4_TIME_APPLY };
enum { P4_TIME_NONE, P4_TIME_START, P4_TIME_STOP };
#define P4_TIME_TIMEOUT_MS 60000LL
#define P4_TIME_RETRY_MS 300000LL
#define P4_TIME_INTERVAL_MS 3600000LL

typedef struct {
    p4_time_sync_snapshot_t status;
    bool online, active;
    int64_t deadline_ms, next_ms;
} p4_time_sync_state_t;

// All scheduling uses monotonic time. Caller serializes the worker and SNTP callback.
int p4_time_sync_step(p4_time_sync_state_t *s, bool online, bool request, int64_t now_ms);
bool p4_time_sync_accepts(const p4_time_sync_state_t *s, int64_t now_ms);
bool p4_time_sync_valid(int64_t seconds, int32_t microseconds);
void p4_time_sync_success(p4_time_sync_state_t *s, uint32_t unix_s, int64_t now_ms);
void p4_time_sync_failure(p4_time_sync_state_t *s, uint32_t error, int64_t now_ms);
