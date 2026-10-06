#!/bin/sh
set -eu
P4PAD_TASK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
P4PAD_TASK_OUT="$P4PAD_TASK_ROOT/.cache/tests/pad-damage"
mkdir -p "$(dirname -- "$P4PAD_TASK_OUT")"
clang -std=c11 -O2 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4PAD_TASK_ROOT/firmware/main" \
  "$P4PAD_TASK_ROOT/firmware/main/pad_damage.c" \
  "$P4PAD_TASK_ROOT/firmware/main/display_pipeline.c" \
  "$P4PAD_TASK_ROOT/tests/c/pad_damage_test.c" -o "$P4PAD_TASK_OUT"
"$P4PAD_TASK_OUT"
