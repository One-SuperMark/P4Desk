#pragma once
#include <stdbool.h>
#include <stdint.h>

// A failed read is not a release. Cancel the current gesture and suppress
// cached coordinates until the controller reports a fresh empty frame.
typedef struct {
    bool waiting_release;
} p4desk_pad_touch_recovery_t;

static inline bool p4desk_pad_touch_sample_allowed(p4desk_pad_touch_recovery_t *guard,
                                                  bool read_ok, bool fresh, uint8_t count)
{
    if (!read_ok) guard->waiting_release = true;
    if (guard->waiting_release) {
        if (!read_ok || !fresh || count) return false;
        guard->waiting_release = false;
    }
    return true;
}
