#include "usb_rx_stream.h"
#include <string.h>

static void received_message(const p4p_header_t *header, const uint8_t *payload, void *context)
{
    p4desk_rx_stream_t *stream = context;
    stream->on_message(header, payload, stream->message_epoch, stream->context);
}

void p4desk_rx_stream_init(p4desk_rx_stream_t *stream, uint8_t *buffer, size_t capacity,
                           p4desk_rx_message_fn on_message, void *context)
{
    memset(stream, 0, sizeof(*stream));
    stream->on_message = on_message;
    stream->context = context;
    p4p_stream_init(&stream->parser, buffer, capacity, received_message, stream);
}

void p4desk_rx_stream_reset(p4desk_rx_stream_t *stream)
{
    p4p_stream_reset(&stream->parser);
    stream->last_rx_us = 0;
    stream->message_epoch = 0;
}

void p4desk_rx_stream_expire(p4desk_rx_stream_t *stream, int64_t now_us)
{
    // A mode change invalidates an old image's epoch, but is not a USB message
    // boundary. Only abandon a partial message after receive bytes are idle.
    // The host's 2-second OUT timeout is shorter than this receive deadline.
    if (stream->parser.header_used && now_us >= stream->last_rx_us &&
        now_us - stream->last_rx_us >= P4DESK_RX_IDLE_TIMEOUT_US)
        p4desk_rx_stream_reset(stream);
}

void p4desk_rx_stream_feed(p4desk_rx_stream_t *stream, const uint8_t *bytes, size_t length,
                           int64_t now_us, uint32_t epoch)
{
    p4desk_rx_stream_expire(stream, now_us);
    if (!length) return;
    stream->last_rx_us = now_us;
    // Fix the epoch when a message header begins. Continue consuming a stale
    // JPEG's declared payload so its tail never becomes a candidate header.
    while (length) {
        if (stream->parser.header_used == 0) stream->message_epoch = epoch;
        size_t count;
        if (stream->parser.header_used < P4P_HEADER_SIZE)
            count = P4P_HEADER_SIZE - stream->parser.header_used;
        else
            count = stream->parser.header.payload_length - stream->parser.payload_used;
        if (count > length) count = length;
        if (!count) { p4p_stream_reset(&stream->parser); continue; }
        p4p_stream_feed(&stream->parser, bytes, count);
        bytes += count;
        length -= count;
    }
}
