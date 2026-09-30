#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$P4DESK_ROOT/.cache/tests"
clang -std=c11 -O2 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/firmware/main" \
  "$P4DESK_ROOT/firmware/main/time_sync_state.c" \
  "$P4DESK_ROOT/tests/c/time_sync_state_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/time-sync"
"$P4DESK_ROOT/.cache/tests/time-sync"
