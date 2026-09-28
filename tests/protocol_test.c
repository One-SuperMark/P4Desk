#include "p4desk_protocol.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned received;
static uint16_t last_sequence;
static void receive(const p4p_header_t *h, const uint8_t *data, void *ctx) {
    (void)ctx; assert(data != NULL); assert(h->payload_length > 0);
    received++; last_sequence = h->sequence;
}
int main(void) {
    assert(p4p_crc16((const uint8_t *)"123456789", 9) == 0x29b1);
    const uint8_t payload[] = "{\"op\":\"hello\",\"request_id\":7,\"version\":1}";
    uint8_t wire[P4P_HEADER_SIZE + sizeof(payload) - 1];
    p4p_header_t h = {.crc16=p4p_crc16(payload,sizeof(payload)-1),.kind=P4P_KIND_CONTROL,
        .flags=P4P_CRC_PRESENT,.sequence=7,.payload_length=sizeof(payload)-1};
    assert(p4p_encode_header(&h,wire)); memcpy(wire+P4P_HEADER_SIZE,payload,sizeof(payload)-1);
    uint8_t buffer[P4P_MAX_CONTROL]; p4p_stream_t stream;
    for (size_t split=0; split <= sizeof(wire); ++split) {
        received=0; p4p_stream_init(&stream,buffer,sizeof(buffer),receive,NULL);
        assert(p4p_stream_feed(&stream,wire,split));
        assert(p4p_stream_feed(&stream,wire+split,sizeof(wire)-split));
        assert(received==1 && last_sequence==7 && stream.errors==0);
    }
    received=0; p4p_stream_init(&stream,buffer,sizeof(buffer),receive,NULL);
    for (size_t i=0; i<sizeof(wire); ++i) assert(p4p_stream_feed(&stream,wire+i,1));
    assert(received==1);
    uint8_t bad[sizeof(wire)]; memcpy(bad,wire,sizeof(wire)); bad[16]^=1;
    assert(!p4p_stream_feed(&stream,bad,sizeof(bad))); assert(received==1);
    assert(p4p_stream_feed(&stream,wire,sizeof(wire))); assert(received==2 && stream.errors==1);
    memset(bad,0xff,sizeof(bad)); received=0; p4p_stream_reset(&stream);
    assert(!p4p_stream_feed(&stream,bad,sizeof(bad)));
    p4p_stream_feed(&stream,wire,sizeof(wire)); assert(received==1);
    h.kind=P4P_KIND_JPEG; h.flags=0; h.x=h.y=0; h.width=1024; h.height=576;
    assert(!p4p_encode_header(&h,wire)); h.height=600; assert(p4p_encode_header(&h,wire));
    h.payload_length=P4P_MAX_JPEG+1; assert(!p4p_encode_header(&h,wire));
    puts("C protocol: CRC, every split, byte stream, corruption recovery, resolution and capacity passed");
    return 0;
}
