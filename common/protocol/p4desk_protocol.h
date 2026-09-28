#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define P4P_VERSION 1
#define P4P_HEADER_SIZE 16
#define P4P_KIND_JPEG 3
#define P4P_KIND_CONTROL 16
#define P4P_KIND_RESOURCE 17
#define P4P_CRC_PRESENT 1
#define P4P_MAX_JPEG (1024U * 1024U)
#define P4P_MAX_CONTROL (64U * 1024U)
#define P4P_MAX_RESOURCE (32768U + 4U)
#define P4P_WIDTH 1024
#define P4P_HEIGHT 600

typedef struct {
    uint16_t crc16;
    uint8_t kind, flags;
    uint16_t x, y, width, height, sequence;
    uint32_t payload_length;
} p4p_header_t;

uint16_t p4p_crc16(const uint8_t *data, size_t length);
bool p4p_decode_header(const uint8_t bytes[P4P_HEADER_SIZE], p4p_header_t *header);
bool p4p_encode_header(const p4p_header_t *header, uint8_t bytes[P4P_HEADER_SIZE]);
typedef void (*p4p_message_fn)(const p4p_header_t *, const uint8_t *, void *);
typedef struct {
    uint8_t header_bytes[P4P_HEADER_SIZE];
    size_t header_used, payload_used, capacity;
    uint8_t *payload;
    p4p_header_t header;
    p4p_message_fn on_message;
    void *context;
    uint32_t errors;
} p4p_stream_t;
void p4p_stream_init(p4p_stream_t *stream, uint8_t *buffer, size_t capacity,
                     p4p_message_fn on_message, void *context);
void p4p_stream_reset(p4p_stream_t *stream);
bool p4p_stream_feed(p4p_stream_t *stream, const uint8_t *bytes, size_t length);
