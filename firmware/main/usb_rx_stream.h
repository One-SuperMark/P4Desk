#pragma once
#include <stdint.h>
#include "p4desk_protocol.h"

#define P4DESK_RX_IDLE_TIMEOUT_US 3000000LL

typedef void (*p4desk_rx_message_fn)(const p4p_header_t *header, const uint8_t *payload,
                                    uint32_t epoch, void *context);
typedef struct {
    p4p_stream_t parser;
    uint32_t message_epoch;
    int64_t last_rx_us;
    p4desk_rx_message_fn on_message;
    void *context;
} p4desk_rx_stream_t;

void p4desk_rx_stream_init(p4desk_rx_stream_t *stream, uint8_t *buffer, size_t capacity,
                           p4desk_rx_message_fn on_message, void *context);
void p4desk_rx_stream_reset(p4desk_rx_stream_t *stream);
void p4desk_rx_stream_expire(p4desk_rx_stream_t *stream, int64_t now_us);
void p4desk_rx_stream_feed(p4desk_rx_stream_t *stream, const uint8_t *bytes, size_t length,
                           int64_t now_us, uint32_t epoch);
