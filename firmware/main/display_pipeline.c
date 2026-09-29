#include "display_pipeline.h"
#include <assert.h>

void p4dp_init(p4dp_owner_t *owner, uint8_t initial_scanning)
{
    assert(owner && initial_scanning < P4DP_BUFFER_COUNT);
    for (int index = 0; index < P4DP_BUFFER_COUNT; index++) owner->states[index] = P4DP_FREE;
    owner->scanning = initial_scanning;
    owner->states[initial_scanning] = P4DP_SCANNING;
    owner->pending = owner->ready = owner->building = -1;
}

int p4dp_reserve(p4dp_owner_t *owner)
{
    if (owner->building >= 0 || owner->ready >= 0) return -1;
    for (int index = 0; index < P4DP_BUFFER_COUNT; index++) {
        if (owner->states[index] == P4DP_FREE) {
            owner->states[index] = P4DP_BUILDING;
            owner->building = (int8_t)index;
            return index;
        }
    }
    return -1;
}

bool p4dp_ready(p4dp_owner_t *owner, uint8_t index)
{
    if (index >= P4DP_BUFFER_COUNT || owner->building != index ||
        owner->states[index] != P4DP_BUILDING || owner->ready >= 0) return false;
    owner->building = -1; owner->ready = (int8_t)index;
    owner->states[index] = P4DP_READY;
    return true;
}

bool p4dp_discard_prepared(p4dp_owner_t *owner, uint8_t index)
{
    if (index >= P4DP_BUFFER_COUNT) return false;
    if (owner->states[index] == P4DP_BUILDING && owner->building == index) owner->building = -1;
    else if (owner->states[index] == P4DP_READY && owner->ready == index) owner->ready = -1;
    else return false;
    owner->states[index] = P4DP_FREE;
    return true;
}

bool p4dp_submit(p4dp_owner_t *owner, uint8_t index)
{
    if (index >= P4DP_BUFFER_COUNT || owner->pending >= 0 || owner->ready != index ||
        owner->states[index] != P4DP_READY) return false;
    owner->ready = -1; owner->pending = (int8_t)index;
    owner->states[index] = P4DP_PENDING;
    return true;
}

bool p4dp_dma_complete(p4dp_owner_t *owner, uint8_t completed, uint8_t next)
{
    if (completed >= P4DP_BUFFER_COUNT || next >= P4DP_BUFFER_COUNT ||
        owner->scanning != completed || owner->states[completed] != P4DP_SCANNING) return false;
    if (completed == next) return true;
    if (owner->pending != next || owner->states[next] != P4DP_PENDING) return false;
    owner->states[completed] = P4DP_FREE;
    owner->states[next] = P4DP_SCANNING;
    owner->scanning = next; owner->pending = -1;
    return true;
}
