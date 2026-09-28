#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "board_p4.h"
#include "p4desk_protocol.h"

void p4desk_runtime_init(board_p4_t *board);
uint32_t p4desk_epoch(void);
void p4desk_receive_message(const p4p_header_t *header, const uint8_t *payload, uint32_t epoch);
void p4desk_usb_mount_changed(bool connected);
void p4desk_usb_init(void);
void p4desk_usb_release_media(void);
uint32_t p4desk_usb_parser_errors(void);
uint32_t p4desk_usb_connection_generation(void);
bool p4desk_usb_queue_json(const char *json, size_t length, uint16_t sequence, bool priority);
bool p4desk_usb_queue_touch(const char *json, size_t length, uint16_t sequence, bool released);
void p4desk_usb_queue_media(uint16_t usage);
