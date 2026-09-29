#!/bin/sh
set -eu
TASK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TASK_OUT="$TASK_ROOT/.cache/tests/display-transition"
mkdir -p "$(dirname -- "$TASK_OUT")"
clang -std=c11 -O2 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$TASK_ROOT/firmware/main" \
  "$TASK_ROOT/firmware/main/display_transition.c" \
  "$TASK_ROOT/tests/c/display_transition_test.c" -o "$TASK_OUT"
"$TASK_OUT"
