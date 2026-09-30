#include "pad_touch_queue.h"

// Internal only; pop restores the public MOVE kind. Preserve the first sample
// beyond tap slop even if the finger returns to its origin before the next frame.
#define MOVE_ANCHOR 0x80000002U

void p4desk_pad_touch_queue_reset(p4desk_pad_touch_queue_t *queue)
{
    if (!queue) return;
    queue->head = 0;
    queue->count = 0;
    queue->gesture_active = false;
    queue->movement_anchored = false;
}

bool p4desk_pad_touch_queue_push(p4desk_pad_touch_queue_t *queue,
                               p4desk_pad_touch_event_t event)
{
    if (!queue) return false;
    if (event.kind == P4DESK_PAD_TOUCH_DOWN) {
        queue->origin_x=event.x;
        queue->origin_y=event.y;
        queue->gesture_active=true;
        queue->movement_anchored=false;
    } else if (event.kind == P4DESK_PAD_TOUCH_MOVE && queue->gesture_active) {
        int32_t dx=event.x-queue->origin_x, dy=event.y-queue->origin_y;
        if (!queue->movement_anchored && (dx>8 || dx< -8 || dy>8 || dy< -8)) {
            event.kind=MOVE_ANCHOR;
            queue->movement_anchored=true;
        }
        if (queue->count) {
            uint32_t last=(queue->head+queue->count-1)%P4DESK_PAD_TOUCH_QUEUE_CAPACITY;
            if (queue->events[last].kind==P4DESK_PAD_TOUCH_MOVE) {
                queue->events[last]=event;
                return true;
            }
        }
    } else if (event.kind == P4DESK_PAD_TOUCH_UP || event.kind == P4DESK_PAD_TOUCH_CANCEL) {
        queue->gesture_active=false;
    }
    if (queue->count == P4DESK_PAD_TOUCH_QUEUE_CAPACITY) {
        // Fail closed. The producer blocks new contacts until all fingers lift;
        // the consumer releases any pressed widget before accepting another tap.
        p4desk_pad_touch_queue_reset(queue);
        queue->events[0] = (p4desk_pad_touch_event_t){.kind = P4DESK_PAD_TOUCH_CANCEL};
        queue->count = 1;
        return false;
    }
    uint32_t tail = (queue->head + queue->count) % P4DESK_PAD_TOUCH_QUEUE_CAPACITY;
    queue->events[tail] = event;
    ++queue->count;
    return true;
}

bool p4desk_pad_touch_queue_pop(p4desk_pad_touch_queue_t *queue,
                              p4desk_pad_touch_event_t *event)
{
    if (!queue || !event || !queue->count) return false;
    *event = queue->events[queue->head];
    if (event->kind==MOVE_ANCHOR) event->kind=P4DESK_PAD_TOUCH_MOVE;
    queue->head = (queue->head + 1) % P4DESK_PAD_TOUCH_QUEUE_CAPACITY;
    --queue->count;
    return true;
}
