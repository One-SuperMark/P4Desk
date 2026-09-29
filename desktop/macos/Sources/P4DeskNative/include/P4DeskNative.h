#pragma once
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// This bridge owns one display per handle. Creation and destruction run on the main thread.
void *P4DisplayCreate(uint32_t width, uint32_t height, char *error, size_t error_capacity);
uint32_t P4DisplayID(void *handle);
void P4DisplayDestroy(void *handle);
bool P4DisplaySymbolsAvailable(void);

// All callbacks are delivered from the session's serial USB queue; bytes are callback-scoped.
// CONNECTED token is the device's negotiated Mbps: 12, 480, 5000, 10000, or 0 (unknown).
// SENT token is the enqueue token. Its 8-byte little-endian metadata is elapsed microseconds
// from the first OUT submission until every byte of that logical packet has completed.
// This excludes the host queue wait, includes partial/chunk completion gaps, and does not
// mean the device has decoded or presented a video frame. Older bridges may omit metadata.
enum { P4USB_CONNECTED = 1, P4USB_DISCONNECTED = 2, P4USB_BYTES = 3,
       P4USB_SENT = 4, P4USB_ERROR = 5 };
typedef void (*P4USBCallback)(void *context, uint32_t event, uint32_t token,
                            const uint8_t *bytes, size_t length, const char *reason);
void *P4USBCreate(P4USBCallback callback, void *context, uint16_t vendor_id, uint16_t product_id);
bool P4USBEnqueue(void *handle, const uint8_t *bytes, size_t length, bool video, uint32_t token);
void P4USBClearVideo(void *handle);
void P4USBReconnect(void *handle);
void P4USBStop(void *handle);

#ifdef __cplusplus
}
#endif
