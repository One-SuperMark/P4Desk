#include "display_transition.h"
#include <string.h>

void p4dt_cancel(p4desk_display_transition_t *s) { memset(s, 0, sizeof(*s)); }
bool p4dt_arm(p4desk_display_transition_t *s, uint32_t duration_ms)
{
    if (!duration_ms || duration_ms > 2000) return false;
    p4dt_cancel(s);
    s->armed = true;
    s->duration_ms = duration_ms;
    return true;
}
void p4dt_bind(p4desk_display_transition_t *s, uint32_t epoch)
{
    if (!s->armed) { p4dt_cancel(s); return; }
    s->armed = false;
    s->active = true;
    s->epoch = epoch;
}
bool p4dt_pending(const p4desk_display_transition_t *s) { return s->armed || s->active; }
bool p4dt_active(const p4desk_display_transition_t *s, uint32_t epoch)
{
    return s->active && s->epoch == epoch;
}
bool p4dt_sample(p4desk_display_transition_t *s, uint32_t epoch, int64_t now_us,
                 uint32_t *elapsed_ms, uint32_t *duration_ms)
{
    if (!p4dt_active(s, epoch)) return false;
    if (!s->started) { s->started = true; s->started_us = now_us; }
    uint64_t elapsed = now_us > s->started_us ? (uint64_t)(now_us - s->started_us) / 1000 : 0;
    *elapsed_ms = elapsed < s->duration_ms ? (uint32_t)elapsed : s->duration_ms;
    *duration_ms = s->duration_ms;
    return true;
}
void p4dt_presented(p4desk_display_transition_t *s, uint32_t epoch, bool unmasked)
{
    if (unmasked && s->started && p4dt_active(s, epoch)) p4dt_cancel(s);
}
