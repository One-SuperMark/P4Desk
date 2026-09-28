#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$P4DESK_ROOT/.cache/tests"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/firmware/main" \
  "$P4DESK_ROOT/firmware/main/display_pixels.c" \
  "$P4DESK_ROOT/firmware/main/tests/display_pixels_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/display_pixels_test"
"$P4DESK_ROOT/.cache/tests/display_pixels_test"
