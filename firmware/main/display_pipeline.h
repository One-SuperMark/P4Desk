#pragma once
#include <stdbool.h>
#include <stdint.h>

#define P4DP_BUFFER_COUNT 3
typedef enum { P4DP_FREE, P4DP_BUILDING, P4DP_READY, P4DP_PENDING, P4DP_SCANNING } p4dp_state_t;
typedef struct {
    p4dp_state_t states[P4DP_BUFFER_COUNT];
    uint8_t scanning;
    int8_t pending;
    int8_t ready;
    int8_t building;
} p4dp_owner_t;

void p4dp_init(p4dp_owner_t *owner, uint8_t initial_scanning);
int p4dp_reserve(p4dp_owner_t *owner);
bool p4dp_ready(p4dp_owner_t *owner, uint8_t index);
bool p4dp_discard_prepared(p4dp_owner_t *owner, uint8_t index);
bool p4dp_submit(p4dp_owner_t *owner, uint8_t index);
/// Only actual DMA completion events may release a scanning buffer.
/// A repeated scan with completed == next retains that buffer's ownership.
bool p4dp_dma_complete(p4dp_owner_t *owner, uint8_t completed, uint8_t next);
