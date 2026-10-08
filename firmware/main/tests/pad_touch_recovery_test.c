#include "pad_touch_recovery.h"
#include "pad_touch_queue.h"
#include <assert.h>
#include <stdio.h>

int main(void)
{
    p4desk_pad_touch_recovery_t guard = {0};
    p4desk_pad_touch_queue_t queue = {0};
    assert(p4desk_pad_touch_sample_allowed(&guard, true, true, 1));
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){.kind = P4DESK_PAD_TOUCH_DOWN}));
    assert(!p4desk_pad_touch_sample_allowed(&guard, false, false, 0));
    assert(guard.waiting_release);
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){.kind = P4DESK_PAD_TOUCH_CANCEL}));
    // Neither cached emptiness nor a fresh contact is proof that all fingers
    // lifted. Do not turn a bus error into UP and execute the pressed button.
    assert(!p4desk_pad_touch_sample_allowed(&guard, true, false, 0));
    assert(!p4desk_pad_touch_sample_allowed(&guard, true, true, 1));
    assert(!p4desk_pad_touch_sample_allowed(&guard, false, true, 0));
    assert(p4desk_pad_touch_sample_allowed(&guard, true, true, 0));
    assert(!guard.waiting_release);
    p4desk_pad_touch_event_t event;
    assert(p4desk_pad_touch_queue_pop(&queue, &event) && event.kind == P4DESK_PAD_TOUCH_DOWN);
    assert(p4desk_pad_touch_queue_pop(&queue, &event) && event.kind == P4DESK_PAD_TOUCH_CANCEL);
    assert(!p4desk_pad_touch_queue_pop(&queue, &event));
    // After the release, another complete tap retains its normal boundaries.
    assert(p4desk_pad_touch_sample_allowed(&guard, true, true, 1));
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){.kind = P4DESK_PAD_TOUCH_DOWN}));
    assert(p4desk_pad_touch_sample_allowed(&guard, true, true, 0));
    assert(p4desk_pad_touch_queue_push(&queue, (p4desk_pad_touch_event_t){.kind = P4DESK_PAD_TOUCH_UP}));
    assert(p4desk_pad_touch_queue_pop(&queue, &event) && event.kind == P4DESK_PAD_TOUCH_DOWN);
    assert(p4desk_pad_touch_queue_pop(&queue, &event) && event.kind == P4DESK_PAD_TOUCH_UP);
    puts("Pad touch recovery: error cancellation, fresh release and next tap passed");
}
