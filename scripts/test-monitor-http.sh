#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$P4DESK_ROOT/.cache/tests"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/firmware/main" \
  "$P4DESK_ROOT/firmware/main/tests/monitor_http_body_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/monitor_http_body_test"
"$P4DESK_ROOT/.cache/tests/monitor_http_body_test"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/firmware/main/tests/monitor_http_stubs" \
  -I "$P4DESK_ROOT/firmware/main" \
  "$P4DESK_ROOT/firmware/main/tests/monitor_http_session_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/monitor_http_session_test"
"$P4DESK_ROOT/.cache/tests/monitor_http_session_test"
