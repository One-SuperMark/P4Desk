#!/bin/sh
set -eu
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
# Read only public headers from the pinned component, without an LVGL dependency.
# An official component cache can be supplied for a pre-build host check.
P4DESK_FREETYPE_HEADERS=${P4DESK_FREETYPE_HEADERS:-"$P4DESK_ROOT/firmware/managed_components/espressif__freetype/freetype/include"}
test -f "$P4DESK_FREETYPE_HEADERS/ft2build.h"
mkdir -p "$P4DESK_ROOT/.cache/tests"
clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
    -I "$P4DESK_ROOT/firmware/components/board_p4/tests/typeface_stubs" \
    -I "$P4DESK_ROOT/firmware/components/board_p4/include" \
    -I "$P4DESK_FREETYPE_HEADERS" \
    "$P4DESK_ROOT/firmware/components/board_p4/tests/typeface_test.c" \
    -o "$P4DESK_ROOT/.cache/tests/typeface_test"
"$P4DESK_ROOT/.cache/tests/typeface_test"
if [ -n "${P4DESK_FREETYPE_LIBRARY:-}" ]; then
    clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
        -I "$P4DESK_ROOT/firmware/components/board_p4/tests/typeface_stubs" \
        -I "$P4DESK_ROOT/firmware/components/board_p4/include" \
        -I "$P4DESK_FREETYPE_HEADERS" \
        "$P4DESK_ROOT/firmware/components/board_p4/tests/typeface_freetype_smoke.c" \
        "$P4DESK_FREETYPE_LIBRARY" \
        -o "$P4DESK_ROOT/.cache/tests/typeface_freetype_smoke"
    "$P4DESK_ROOT/.cache/tests/typeface_freetype_smoke" "$P4DESK_ROOT/assets/fonts/NotoSansSC-Regular.otf"
    "$P4DESK_ROOT/.cache/tests/typeface_freetype_smoke" "$P4DESK_ROOT/assets/fonts/HarmonyOS_Sans_SC_Regular.ttf"
fi
