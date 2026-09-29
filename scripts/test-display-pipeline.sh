#!/bin/sh
set -eu
TASK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TASK_OUT="$TASK_ROOT/.cache/tests/display-pipeline"
mkdir -p "$(dirname -- "$TASK_OUT")"
clang -std=c11 -O2 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$TASK_ROOT/firmware/main" \
  "$TASK_ROOT/firmware/main/display_pipeline.c" \
  "$TASK_ROOT/tests/c/display_pipeline_test.c" -o "$TASK_OUT"
"$TASK_OUT"
