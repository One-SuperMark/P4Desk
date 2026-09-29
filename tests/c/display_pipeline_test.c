#include <assert.h>
#include <stdio.h>
#include "display_pipeline.h"

static void pipeline_can_prepare_while_scanning_and_present_each_boundary(void)
{
    p4dp_owner_t owner;
    p4dp_init(&owner, 0);
    int first = p4dp_reserve(&owner);
    assert(first == 1 && p4dp_ready(&owner, first) && p4dp_submit(&owner, first));
    int second = p4dp_reserve(&owner);
    assert(second == 2 && p4dp_ready(&owner, second));
    assert(p4dp_reserve(&owner) < 0);
    assert(!p4dp_submit(&owner, second)); // First requested frame has not started yet.
    for (int refresh = 0; refresh < 1000; refresh++) {
        int completed = owner.scanning, next = owner.pending;
        assert(completed >= 0 && next >= 0 && completed != next);
        assert(p4dp_dma_complete(&owner, completed, next));
        assert(owner.states[completed] == P4DP_FREE);
        assert(owner.states[next] == P4DP_SCANNING);
        int prepared = owner.ready;
        assert(prepared >= 0 && p4dp_submit(&owner, prepared));
        assert(owner.states[prepared] == P4DP_PENDING);
        int building = p4dp_reserve(&owner);
        assert(building == completed); // Only the actual completed source can be written again.
        assert(p4dp_ready(&owner, building));
    }
}

static void repeated_or_invalid_callbacks_never_release_scanning_or_pending(void)
{
    p4dp_owner_t owner;
    p4dp_init(&owner, 0);
    int prepared = p4dp_reserve(&owner);
    assert(p4dp_ready(&owner, prepared) && p4dp_submit(&owner, prepared));
    for (int repeat = 0; repeat < 100; repeat++) {
        assert(p4dp_dma_complete(&owner, 0, 0));
        assert(owner.states[0] == P4DP_SCANNING);
        assert(owner.states[prepared] == P4DP_PENDING);
    }
    assert(!p4dp_dma_complete(&owner, prepared, prepared));
    assert(!p4dp_dma_complete(&owner, 0, 2));
    assert(!p4dp_discard_prepared(&owner, 0));
    assert(!p4dp_discard_prepared(&owner, prepared));
    assert(p4dp_dma_complete(&owner, 0, prepared));
    assert(!p4dp_dma_complete(&owner, 0, prepared)); // Late duplicate cannot free the new scan.
    assert(owner.states[prepared] == P4DP_SCANNING);
}

static void mode_change_cancels_only_unsubmitted_buffers(void)
{
    p4dp_owner_t owner;
    p4dp_init(&owner, 0);
    int pending = p4dp_reserve(&owner);
    assert(p4dp_ready(&owner, pending) && p4dp_submit(&owner, pending));
    int building = p4dp_reserve(&owner);
    assert(building == 2 && p4dp_discard_prepared(&owner, building));
    building = p4dp_reserve(&owner);
    assert(p4dp_ready(&owner, building) && p4dp_discard_prepared(&owner, building));
    assert(!p4dp_discard_prepared(&owner, pending));
    assert(p4dp_dma_complete(&owner, 0, pending));
    int pad = p4dp_reserve(&owner);
    assert(pad >= 0 && pad != pending && p4dp_ready(&owner, pad) && p4dp_submit(&owner, pad));
    assert(p4dp_dma_complete(&owner, pending, pad));
    assert(owner.states[pending] == P4DP_FREE && owner.states[pad] == P4DP_SCANNING);
}

static void pad_can_be_prepared_before_old_pending_starts(void)
{
    p4dp_owner_t owner;
    p4dp_init(&owner, 0);
    assert(p4dp_ready(&owner, p4dp_reserve(&owner)) && p4dp_submit(&owner, 1));
    assert(p4dp_ready(&owner, p4dp_reserve(&owner)) && owner.ready == 2);
    assert(p4dp_discard_prepared(&owner, 2));
    int pad = p4dp_reserve(&owner);
    assert(pad == 2 && p4dp_ready(&owner, pad));
    assert(!p4dp_submit(&owner, pad));
    assert(p4dp_dma_complete(&owner, 0, 0));
    assert(owner.pending == 1 && owner.ready == 2 && owner.states[0] == P4DP_SCANNING);
    assert(p4dp_dma_complete(&owner, 0, 1));
    assert(p4dp_submit(&owner, pad));
    assert(p4dp_dma_complete(&owner, 1, pad));
    assert(owner.states[1] == P4DP_FREE && owner.states[pad] == P4DP_SCANNING);
}

int main(void)
{
    pipeline_can_prepare_while_scanning_and_present_each_boundary();
    repeated_or_invalid_callbacks_never_release_scanning_or_pending();
    mode_change_cancels_only_unsubmitted_buffers();
    pad_can_be_prepared_before_old_pending_starts();
    puts("display pipeline ownership and mode transition tests passed");
}
