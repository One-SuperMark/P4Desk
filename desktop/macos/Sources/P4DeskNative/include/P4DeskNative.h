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

// All callbacks are delivered from the session's serial USB queue. BYTES are callback-scoped.
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
