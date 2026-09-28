#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$P4DESK_ROOT/.cache/tests"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/common/protocol" \
  "$P4DESK_ROOT/common/protocol/p4desk_protocol.c" "$P4DESK_ROOT/tests/protocol_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/protocol_test"
"$P4DESK_ROOT/.cache/tests/protocol_test"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/common/protocol" -I "$P4DESK_ROOT/firmware/main" \
  "$P4DESK_ROOT/common/protocol/p4desk_protocol.c" \
  "$P4DESK_ROOT/firmware/main/usb_rx_stream.c" \
  "$P4DESK_ROOT/firmware/main/tests/usb_rx_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/usb_rx_test"
"$P4DESK_ROOT/.cache/tests/usb_rx_test"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I "$P4DESK_ROOT/firmware/main" \
  "$P4DESK_ROOT/firmware/main/usb_tx_queue.c" \
  "$P4DESK_ROOT/firmware/main/touch_exit_notice.c" \
  "$P4DESK_ROOT/firmware/main/tests/usb_tx_test.c" \
  -o "$P4DESK_ROOT/.cache/tests/usb_tx_test"
"$P4DESK_ROOT/.cache/tests/usb_tx_test"
cd "$P4DESK_ROOT"
cargo test -p p4desk-protocol
