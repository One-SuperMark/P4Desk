#include "usb_tx_queue.h"

bool p4desk_tx_queue_put(p4desk_tx_queue_t *queue, p4desk_tx_packet_t packet, bool replace_oldest)
{
    if (queue->send(queue->context, &packet)) return true;
    if (replace_oldest) {
        p4desk_tx_packet_t previous;
        // Producer and consumer may race to receive: only the successful
        // receiver owns/frees the old pointer. A sole producer can then retry.
        if (queue->receive(queue->context, &previous)) queue->dispose(previous.bytes);
        if (queue->send(queue->context, &packet)) return true;
    }
    queue->dispose(packet.bytes);
    return false;
}

void p4desk_tx_queue_clear(p4desk_tx_queue_t *queue)
{
    p4desk_tx_packet_t packet;
    while (queue->receive(queue->context, &packet)) queue->dispose(packet.bytes);
}

bool p4desk_tx_select_next(p4desk_tx_queue_t *release, p4desk_tx_queue_t *priority,
                           p4desk_tx_queue_t *events, uint32_t generation, p4desk_tx_packet_t *active)
{
    if (active->bytes) return true;
    for (;;) {
        p4desk_tx_queue_t *source;
        if (release->receive(release->context, active)) source = release;
        else if (priority->receive(priority->context, active)) source = priority;
        else if (events->receive(events->context, active)) source = events;
        else return false;
        if (active->generation == generation) return true;
        // A producer can resume after disconnect cleanup. Its late old packet
        // is still owned, but must never execute in the next USB connection.
        source->dispose(active->bytes);
        *active = (p4desk_tx_packet_t){0};
    }
}
