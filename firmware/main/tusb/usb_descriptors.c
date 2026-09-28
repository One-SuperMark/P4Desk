/*
 * SPDX-License-Identifier: MIT
 * Descriptor structure follows TinyUSB's MIT-licensed device examples.
 */
#include <string.h>
#include "tusb.h"

enum { ITF_VENDOR = 0, ITF_HID, ITF_COUNT };
enum { STR_MANUFACTURER = 1, STR_PRODUCT, STR_VENDOR, STR_HID };
enum { EP_VENDOR_OUT = 0x01, EP_VENDOR_IN = 0x81, EP_HID_IN = 0x82 };

static const tusb_desc_device_t s_device = {
    .bLength = sizeof(tusb_desc_device_t), .bDescriptorType = TUSB_DESC_DEVICE,
    .bcdUSB = 0x0200, .bDeviceClass = 0, .bDeviceSubClass = 0, .bDeviceProtocol = 0,
    .bMaxPacketSize0 = CFG_TUD_ENDPOINT0_SIZE, .idVendor = 0x303A, .idProduct = 0x4044,
    .bcdDevice = 0x0100, .iManufacturer = STR_MANUFACTURER, .iProduct = STR_PRODUCT,
    .iSerialNumber = 0, .bNumConfigurations = 1,
};

static const uint8_t s_hid_report[] = {
    TUD_HID_REPORT_DESC_CONSUMER(HID_REPORT_ID(1))
};
#define CONFIG_LENGTH (TUD_CONFIG_DESC_LEN + TUD_VENDOR_DESC_LEN + TUD_HID_DESC_LEN)
#define CONFIG_DESCRIPTOR(descriptor_type, bulk_size, hid_interval) \
    TUD_CONFIG_DESCRIPTOR(1, ITF_COUNT, 0, CONFIG_LENGTH, 0, 500), \
    TUD_VENDOR_DESCRIPTOR(ITF_VENDOR, STR_VENDOR, EP_VENDOR_OUT, EP_VENDOR_IN, bulk_size), \
    TUD_HID_DESCRIPTOR(ITF_HID, STR_HID, HID_ITF_PROTOCOL_NONE, sizeof(s_hid_report), EP_HID_IN, 64, hid_interval)

static const uint8_t s_config_fs[] = {CONFIG_DESCRIPTOR(TUSB_DESC_CONFIGURATION, 64, 8)};
static const uint8_t s_config_hs[] = {CONFIG_DESCRIPTOR(TUSB_DESC_CONFIGURATION, 512, 4)};
static uint8_t s_other_speed[CONFIG_LENGTH];

static const tusb_desc_device_qualifier_t s_qualifier = {
    .bLength = sizeof(tusb_desc_device_qualifier_t), .bDescriptorType = TUSB_DESC_DEVICE_QUALIFIER,
    .bcdUSB = 0x0200, .bDeviceClass = 0, .bDeviceSubClass = 0, .bDeviceProtocol = 0,
    .bMaxPacketSize0 = CFG_TUD_ENDPOINT0_SIZE, .bNumConfigurations = 1, .bReserved = 0,
};

const uint8_t *tud_descriptor_device_cb(void) { return (const uint8_t *)&s_device; }
const uint8_t *tud_descriptor_configuration_cb(uint8_t index)
{
    (void)index;
    return tud_speed_get() == TUSB_SPEED_HIGH ? s_config_hs : s_config_fs;
}
const uint8_t *tud_descriptor_device_qualifier_cb(void) { return (const uint8_t *)&s_qualifier; }
const uint8_t *tud_descriptor_other_speed_configuration_cb(uint8_t index)
{
    (void)index;
    const uint8_t *source = tud_speed_get() == TUSB_SPEED_HIGH ? s_config_fs : s_config_hs;
    memcpy(s_other_speed, source, sizeof(s_other_speed));
    s_other_speed[1] = TUSB_DESC_OTHER_SPEED_CONFIG;
    return s_other_speed;
}
const uint8_t *tud_hid_descriptor_report_cb(uint8_t instance)
{
    (void)instance;
    return s_hid_report;
}
const uint16_t *tud_descriptor_string_cb(uint8_t index, uint16_t langid)
{
    (void)langid;
    static uint16_t text[32];
    static const char *const strings[] = {NULL, "P4Desk", "P4Desk", "P4Desk USB", "P4Desk Media"};
    if (index == 0) {
        text[0] = (TUSB_DESC_STRING << 8) | 4;
        text[1] = 0x0409;
        return text;
    }
    if (index >= sizeof(strings) / sizeof(strings[0])) return NULL;
    size_t len = strlen(strings[index]);
    if (len > 31) len = 31;
    for (size_t n = 0; n < len; n++) text[n + 1] = (uint8_t)strings[index][n];
    text[0] = (TUSB_DESC_STRING << 8) | (2 * len + 2);
    return text;
}
