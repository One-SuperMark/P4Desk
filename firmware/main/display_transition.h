#pragma once
#include <stdbool.h>
#include <stdint.h>

// Pure lifecycle helper; caller serializes access with the display state lock.
typedef struct {
    bool armed, active, started;
    uint32_t epoch, duration_ms;
    int64_t started_us;
} p4desk_display_transition_t;

bool p4dt_arm(p4desk_display_transition_t *state, uint32_t duration_ms);
void p4dt_cancel(p4desk_display_transition_t *state);
void p4dt_bind(p4desk_display_transition_t *state, uint32_t epoch);
bool p4dt_pending(const p4desk_display_transition_t *state);
bool p4dt_active(const p4desk_display_transition_t *state, uint32_t epoch);
// Call only after a valid JPEG has been fully decoded into a BUILDING buffer.
bool p4dt_sample(p4desk_display_transition_t *state, uint32_t epoch, int64_t now_us,
                 uint32_t *elapsed_ms, uint32_t *duration_ms);
// Release input only after an unmasked frame completes LCD DMA.
void p4dt_presented(p4desk_display_transition_t *state, uint32_t epoch, bool unmasked);
