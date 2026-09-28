#include "p4desk_protocol.h"
#include <string.h>

static uint16_t rd16(const uint8_t *p) { return (uint16_t)p[0] | (uint16_t)p[1] << 8; }
static uint32_t rd32(const uint8_t *p) { return (uint32_t)rd16(p) | (uint32_t)rd16(p + 2) << 16; }
static void wr16(uint8_t *p, uint16_t v) { p[0] = (uint8_t)v; p[1] = (uint8_t)(v >> 8); }
static void wr32(uint8_t *p, uint32_t v) { wr16(p, (uint16_t)v); wr16(p + 2, (uint16_t)(v >> 16)); }

uint16_t p4p_crc16(const uint8_t *data, size_t length) {
    uint16_t crc = 0xffff;
    for (size_t i = 0; i < length; ++i) {
        crc ^= (uint16_t)data[i] << 8;
        for (int bit = 0; bit < 8; ++bit)
            crc = (uint16_t)((crc & 0x8000) ? (crc << 1) ^ 0x1021 : crc << 1);
    }
    return crc;
}

static bool valid(const p4p_header_t *h) {
    if (!h || !h->payload_length || h->sequence > 1023 || (h->flags & ~P4P_CRC_PRESENT)) return false;
    if (h->kind == P4P_KIND_JPEG)
        return h->x == 0 && h->y == 0 && h->width == P4P_WIDTH && h->height == P4P_HEIGHT
            && h->payload_length <= P4P_MAX_JPEG;
    if (h->x || h->y || h->width || h->height || h->flags != P4P_CRC_PRESENT) return false;
    if (h->kind == P4P_KIND_CONTROL) return h->payload_length <= P4P_MAX_CONTROL;
    if (h->kind == P4P_KIND_RESOURCE) return h->payload_length >= 4 && h->payload_length <= P4P_MAX_RESOURCE;
    return false;
}

bool p4p_decode_header(const uint8_t bytes[P4P_HEADER_SIZE], p4p_header_t *h) {
    if (!bytes || !h) return false;
    h->crc16 = rd16(bytes); h->kind = bytes[2]; h->flags = bytes[3];
    h->x = rd16(bytes + 4); h->y = rd16(bytes + 6);
    h->width = rd16(bytes + 8); h->height = rd16(bytes + 10);
    uint32_t packed = rd32(bytes + 12);
    h->sequence = packed & 1023; h->payload_length = packed >> 10;
    return valid(h);
}

bool p4p_encode_header(const p4p_header_t *h, uint8_t bytes[P4P_HEADER_SIZE]) {
    if (!bytes || !valid(h)) return false;
    wr16(bytes, h->crc16); bytes[2] = h->kind; bytes[3] = h->flags;
    wr16(bytes + 4, h->x); wr16(bytes + 6, h->y);
    wr16(bytes + 8, h->width); wr16(bytes + 10, h->height);
    wr32(bytes + 12, h->payload_length << 10 | h->sequence);
    return true;
}

void p4p_stream_init(p4p_stream_t *s, uint8_t *buffer, size_t capacity,
                     p4p_message_fn fn, void *context) {
    memset(s, 0, sizeof(*s)); s->payload = buffer; s->capacity = capacity;
    s->on_message = fn; s->context = context;
}
void p4p_stream_reset(p4p_stream_t *s) {
    s->header_used = 0; s->payload_used = 0; memset(&s->header, 0, sizeof(s->header));
}
bool p4p_stream_feed(p4p_stream_t *s, const uint8_t *bytes, size_t length) {
    if (!s || (!bytes && length) || !s->payload || !s->on_message) return false;
    bool ok = true;
    while (length) {
        if (s->header_used < P4P_HEADER_SIZE) {
            size_t n = P4P_HEADER_SIZE - s->header_used; if (n > length) n = length;
            memcpy(s->header_bytes + s->header_used, bytes, n);
            bytes += n; length -= n; s->header_used += n;
            if (s->header_used < P4P_HEADER_SIZE) continue;
            if (!p4p_decode_header(s->header_bytes, &s->header) || s->header.payload_length > s->capacity) {
                s->errors++; ok = false;
                memmove(s->header_bytes, s->header_bytes + 1, P4P_HEADER_SIZE - 1);
                s->header_used = P4P_HEADER_SIZE - 1; s->payload_used = 0;
                continue;
            }
        }
        size_t n = s->header.payload_length - s->payload_used; if (n > length) n = length;
        memcpy(s->payload + s->payload_used, bytes, n);
        s->payload_used += n; bytes += n; length -= n;
        if (s->payload_used == s->header.payload_length) {
            if ((s->header.flags & P4P_CRC_PRESENT) && p4p_crc16(s->payload, s->payload_used) != s->header.crc16) {
                s->errors++; ok = false;
            } else s->on_message(&s->header, s->payload, s->context);
            p4p_stream_reset(s);
        }
    }
    return ok;
}
