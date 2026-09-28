#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t *bytes;
    size_t length;
    uint32_t generation;
} p4desk_tx_packet_t;

typedef struct {
    void *context;
    bool (*send)(void *context, const p4desk_tx_packet_t *packet);
    bool (*receive)(void *context, p4desk_tx_packet_t *packet);
    void (*dispose)(void *bytes);
} p4desk_tx_queue_t;

// Takes ownership in every case. A successful receive transfers it to caller.
bool p4desk_tx_queue_put(p4desk_tx_queue_t *queue, p4desk_tx_packet_t packet, bool replace_oldest);
void p4desk_tx_queue_clear(p4desk_tx_queue_t *queue);
// Retains an active logical message; release preempts only the next selection.
bool p4desk_tx_select_next(p4desk_tx_queue_t *release, p4desk_tx_queue_t *priority,
                           p4desk_tx_queue_t *events, uint32_t generation, p4desk_tx_packet_t *active);
