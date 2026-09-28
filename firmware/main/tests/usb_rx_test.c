#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "usb_rx_stream.h"

typedef struct {
    unsigned count;
    uint8_t kind[4];
    uint16_t sequence[4];
    uint32_t epoch[4];
} observed_t;

static void observed(const p4p_header_t *header, const uint8_t *payload,
                     uint32_t epoch, void *context)
{
    (void)payload;
    observed_t *result = context;
    assert(result->count < 4);
    unsigned index = result->count++;
    result->kind[index] = header->kind;
    result->sequence[index] = header->sequence;
    result->epoch[index] = epoch;
}

int main(void)
{
    static const uint8_t control_payload[] = "{\"op\":\"heartbeat\",\"request_id\":7}";
    uint8_t control[P4P_HEADER_SIZE + sizeof(control_payload) - 1];
    p4p_header_t control_header = {
        .kind = P4P_KIND_CONTROL, .flags = P4P_CRC_PRESENT, .sequence = 7,
        .payload_length = sizeof(control_payload) - 1,
        .crc16 = p4p_crc16(control_payload, sizeof(control_payload) - 1),
    };
    assert(p4p_encode_header(&control_header, control));
    memcpy(control + P4P_HEADER_SIZE, control_payload, sizeof(control_payload) - 1);

    // Include a valid control message inside the opaque image payload. A reset
    // during that payload could incorrectly accept it as a new command.
    uint8_t jpeg[P4P_HEADER_SIZE + 256];
    p4p_header_t jpeg_header = {
        .kind = P4P_KIND_JPEG, .sequence = 10, .width = 1024, .height = 600,
        .payload_length = 256,
    };
    assert(p4p_encode_header(&jpeg_header, jpeg));
    memset(jpeg + P4P_HEADER_SIZE, 0xa5, 256);
    memcpy(jpeg + P4P_HEADER_SIZE + 32, control, sizeof(control));
    uint8_t wire[sizeof(jpeg) + sizeof(control)];
    memcpy(wire, jpeg, sizeof(jpeg));
    memcpy(wire + sizeof(jpeg), control, sizeof(control));

    uint8_t buffer[1024];
    p4desk_rx_stream_t stream;
    for (size_t cut = 1; cut < sizeof(jpeg); cut++) {
        observed_t result = {0};
        p4desk_rx_stream_init(&stream, buffer, sizeof(buffer), observed, &result);
        p4desk_rx_stream_feed(&stream, wire, cut, 1000, 11);
        // The display epoch changes while the host continues its current USB
        // message. The image retains 11; the following actual control gets 12.
        p4desk_rx_stream_feed(&stream, wire + cut, sizeof(wire) - cut, 2000, 12);
        assert(result.count == 2);
        assert(result.kind[0] == P4P_KIND_JPEG && result.epoch[0] == 11);
        assert(result.kind[1] == P4P_KIND_CONTROL && result.epoch[1] == 12);
        assert(result.sequence[0] == 10 && result.sequence[1] == 7);
    }

    observed_t result = {0};
    p4desk_rx_stream_init(&stream, buffer, sizeof(buffer), observed, &result);
    p4desk_rx_stream_feed(&stream, jpeg, 30, 1000, 1);
    p4desk_rx_stream_expire(&stream, 1000 + P4DESK_RX_IDLE_TIMEOUT_US - 1);
    assert(stream.parser.header_used == P4P_HEADER_SIZE);
    p4desk_rx_stream_expire(&stream, 1000 + P4DESK_RX_IDLE_TIMEOUT_US);
    assert(stream.parser.header_used == 0);
    p4desk_rx_stream_feed(&stream, control, sizeof(control), 4000000, 2);
    assert(result.count == 1 && result.kind[0] == P4P_KIND_CONTROL && result.epoch[0] == 2);

    // Expiration also happens before feeding new bytes, even if no periodic
    // task ran while the interface was closed by the host.
    result = (observed_t){0};
    p4desk_rx_stream_init(&stream, buffer, sizeof(buffer), observed, &result);
    p4desk_rx_stream_feed(&stream, jpeg, 5, 1000, 1);
    p4desk_rx_stream_feed(&stream, control, sizeof(control), 4000000, 3);
    assert(result.count == 1 && result.kind[0] == P4P_KIND_CONTROL && result.epoch[0] == 3);

    // Total receive time may exceed 3 seconds: only a gap between bytes expires
    // an incomplete message, so a slowly progressing transfer stays framed.
    result = (observed_t){0};
    p4desk_rx_stream_init(&stream, buffer, sizeof(buffer), observed, &result);
    for (size_t index = 0; index < sizeof(jpeg); index++)
        p4desk_rx_stream_feed(&stream, jpeg + index, 1, 1000 + index * 500000LL, 4);
    assert(result.count == 1 && result.kind[0] == P4P_KIND_JPEG && result.epoch[0] == 4);

    result = (observed_t){0};
    p4desk_rx_stream_init(&stream, buffer, sizeof(buffer), observed, &result);
    p4desk_rx_stream_feed(&stream, jpeg, 30, 1000, 4);
    p4desk_rx_stream_reset(&stream); // Physical disconnect/suspend is a boundary.
    p4desk_rx_stream_feed(&stream, control, sizeof(control), 2000, 5);
    assert(result.count == 1 && result.epoch[0] == 5);
    puts("USB receive lifecycle tests passed");
    return 0;
}
