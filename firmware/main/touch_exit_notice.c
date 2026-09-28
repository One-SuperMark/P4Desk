#include "touch_exit_notice.h"

void p4desk_exit_notice_begin(p4desk_exit_notice_t *notice, uint32_t pad_epoch,
                             uint32_t connection_generation)
{
    notice->pending = true;
    notice->pad_epoch = pad_epoch;
    notice->connection_generation = connection_generation;
}

void p4desk_exit_notice_service(p4desk_exit_notice_t *notice, bool connected, bool pad_mode,
                               uint32_t epoch, uint32_t connection_generation,
                               bool (*send_notice)(void *context), void *context)
{
    if (!notice->pending) return;
    if (!connected || !pad_mode || epoch != notice->pad_epoch ||
        connection_generation != notice->connection_generation) {
        notice->pending = false;
        return;
    }
    if (send_notice(context)) notice->pending = false;
}
