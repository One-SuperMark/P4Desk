#pragma once
#include <stdbool.h>
#include <stdint.h>

typedef struct {
    bool pending;
    uint32_t pad_epoch;
    uint32_t connection_generation;
} p4desk_exit_notice_t;

void p4desk_exit_notice_begin(p4desk_exit_notice_t *notice, uint32_t pad_epoch,
                             uint32_t connection_generation);
void p4desk_exit_notice_service(p4desk_exit_notice_t *notice, bool connected, bool pad_mode,
                               uint32_t epoch, uint32_t connection_generation,
                               bool (*send_notice)(void *context), void *context);
