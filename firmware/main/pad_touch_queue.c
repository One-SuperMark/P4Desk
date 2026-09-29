#include "pad_touch_queue.h"

void p4desk_pad_touch_queue_reset(p4desk_pad_touch_queue_t *queue)
{
    if (!queue) return;
    queue->head = 0;
    queue->count = 0;
}

bool p4desk_pad_touch_queue_push(p4desk_pad_touch_queue_t *queue,
                               p4desk_pad_touch_event_t event)
{
    if (!queue) return false;
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
    queue->head = (queue->head + 1) % P4DESK_PAD_TOUCH_QUEUE_CAPACITY;
    --queue->count;
    return true;
}
