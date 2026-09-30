#pragma once
#include "time_sync_state.h"

// Called only by the radio worker; no additional task, socket wait or UI blocking.
p4_time_sync_snapshot_t p4desk_time_sync_poll(bool online, bool request);
