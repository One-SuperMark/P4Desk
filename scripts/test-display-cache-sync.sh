#!/bin/sh
set -eu
P4CACHE_TASK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
P4CACHE_TASK_OUT="$P4CACHE_TASK_ROOT/.cache/tests/display-cache-sync"
mkdir -p "$(dirname -- "$P4CACHE_TASK_OUT")"
clang -std=c11 -O2 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4CACHE_TASK_ROOT/firmware/main" \
  "$P4CACHE_TASK_ROOT/tests/c/display_cache_sync_test.c" -o "$P4CACHE_TASK_OUT"
"$P4CACHE_TASK_OUT"
