#include "pad_touch_queue.h"
#include <assert.h>
#include <stdio.h>

static void expect(p4desk_pad_touch_queue_t *queue, uint32_t kind, int32_t x, int32_t y)
{
    p4desk_pad_touch_event_t event;
    assert(p4desk_pad_touch_queue_pop(queue, &event));
    assert(event.kind == kind && event.x == x && event.y == y);
}

int main(void)
{
    p4desk_pad_touch_queue_t queue = {0};
    p4desk_pad_touch_event_t event;
    // A quick tap during a long redraw still yields both boundaries, in order.
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){1, 151, 177}));
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){3, 151, 177}));
    expect(&queue, 1, 151, 177);
    expect(&queue, 3, 151, 177);
    assert(!p4desk_pad_touch_queue_pop(&queue, &event));
    // Preserve excursion history; never collapse an out-and-back drag into a tap.
    for (unsigned n = 0; n < 100; ++n) {
        assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){1, 392, 177}));
        assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){2, 392, 403}));
        assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){2, 392, 177}));
        assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){3, 392, 177}));
        expect(&queue, 1, 392, 177);
        expect(&queue, 2, 392, 403);
        expect(&queue, 2, 392, 177);
        expect(&queue, 3, 392, 177);
    }
    // Overflow cancels a gesture, rather than emitting a release without its Down.
    for (unsigned n = 0; n < P4DESK_PAD_TOUCH_QUEUE_CAPACITY; ++n)
        assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){2, (int32_t)n, 100}));
    assert(!p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){3, 99, 100}));
    expect(&queue, P4DESK_PAD_TOUCH_CANCEL, 0, 0);
    assert(!p4desk_pad_touch_queue_pop(&queue, &event));
    // Mode reset discards old contacts. Invalid pointers cannot consume an event.
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){1, 10, 20}));
    assert(!p4desk_pad_touch_queue_pop(&queue, NULL));
    expect(&queue, 1, 10, 20);
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){1, 10, 20}));
    p4desk_pad_touch_queue_reset(&queue);
    assert(!p4desk_pad_touch_queue_pop(&queue, &event));
    assert(!p4desk_pad_touch_queue_push(NULL, (p4desk_pad_touch_event_t){0}));
    assert(!p4desk_pad_touch_queue_pop(NULL, &event));
    p4desk_pad_touch_queue_reset(NULL);
    puts("Pad touch queue: quick tap, drag history, wraparound, overflow cancel and reset passed");
    return 0;
}
