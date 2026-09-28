// TinyUSB's upstream example configuration is MIT licensed.
#pragma once
#include "sdkconfig.h"

#ifndef CFG_TUSB_MCU
#error CFG_TUSB_MCU must be supplied by the TinyUSB component
#endif
#define CFG_TUSB_RHPORT1_MODE (OPT_MODE_DEVICE | OPT_MODE_HIGH_SPEED)
#define CFG_TUSB_OS OPT_OS_FREERTOS
#define CFG_TUSB_OS_INC_PATH freertos/
#define CFG_TUSB_DEBUG 0
#define CFG_TUD_ENABLED 1
#define CFG_TUD_ENDPOINT0_SIZE 64
#define CFG_TUSB_MEM_ALIGN __attribute__((aligned(64)))

#define CFG_TUD_VENDOR 1
#define CFG_TUD_VENDOR_RX_BUFSIZE 8192
#define CFG_TUD_VENDOR_TX_BUFSIZE 8192
#define CFG_TUD_VENDOR_EPSIZE 512
#define CFG_TUD_HID 1
#define CFG_TUD_HID_EP_BUFSIZE 64
#define CFG_TUD_CDC 0
#define CFG_TUD_MSC 0
#define CFG_TUD_MIDI 0
#define CFG_TUD_AUDIO 0
#define CFG_TUD_VIDEO 0
#define CFG_TUD_DFU 0
#define CFG_TUD_DFU_RUNTIME 0
