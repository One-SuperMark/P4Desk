#!/bin/sh
set -eu
TASK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TASK_OUT="$TASK_ROOT/.cache/tests/display-scan-health"
mkdir -p "$(dirname -- "$TASK_OUT")"
clang -std=c11 -O2 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$TASK_ROOT/firmware/main" \
  "$TASK_ROOT/firmware/main/display_scan_health.c" \
  "$TASK_ROOT/firmware/tests/test_display_scan_health.c" -o "$TASK_OUT"
"$TASK_OUT"
