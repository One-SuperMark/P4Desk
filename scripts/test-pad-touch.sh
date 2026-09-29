#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$P4DESK_ROOT/.cache/tests"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/firmware/main" -I "$P4DESK_ROOT/include" \
  "$P4DESK_ROOT/firmware/main/pad_touch_queue.c" \
  "$P4DESK_ROOT/firmware/main/tests/pad_touch_queue_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/pad_touch_queue_test"
"$P4DESK_ROOT/.cache/tests/pad_touch_queue_test"
