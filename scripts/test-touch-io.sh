#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
P4DESK_IDF=/Volumes/work/esp/esp-idf-v6.0.2
mkdir -p "$P4DESK_ROOT/.cache/tests"
# Use IDF's actual public IO vtable, with only platform types and I2C calls stubbed.
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
    -I "$P4DESK_ROOT/firmware/components/board_p4/tests/touch_io_stubs" \
    -I "$P4DESK_ROOT/firmware/components/board_p4/include" \
    -I "$P4DESK_IDF/components/esp_lcd/interface" \
    "$P4DESK_ROOT/firmware/components/board_p4/touch_io.c" \
    "$P4DESK_ROOT/firmware/components/board_p4/tests/touch_io_test.c" \
    -o "$P4DESK_ROOT/.cache/tests/touch_io_test"
"$P4DESK_ROOT/.cache/tests/touch_io_test"
