#pragma once
#include "p4desk_hal.h"

#define P4DESK_PAD_TOUCH_QUEUE_CAPACITY 64U

// The caller holds s_state_lock for both producer and consumer operations.
// Preserve gesture boundaries and movement: a redraw must not lose a quick tap
// or turn a drag out and back into a tap.
typedef struct {
    p4desk_pad_touch_event_t events[P4DESK_PAD_TOUCH_QUEUE_CAPACITY];
    uint32_t head, count;
    int32_t origin_x, origin_y;
    bool gesture_active, movement_anchored;
} p4desk_pad_touch_queue_t;

void p4desk_pad_touch_queue_reset(p4desk_pad_touch_queue_t *queue);
bool p4desk_pad_touch_queue_push(p4desk_pad_touch_queue_t *queue,
                               p4desk_pad_touch_event_t event);
bool p4desk_pad_touch_queue_pop(p4desk_pad_touch_queue_t *queue,
                              p4desk_pad_touch_event_t *event);
