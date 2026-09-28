#!/bin/bash
set -eo pipefail
P4DESK_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
P4DESK_IDF_PATH=${P4DESK_IDF_PATH:-/Volumes/work/esp/esp-idf-v6.0.2}
if [[ ! -f "$P4DESK_IDF_PATH/export.sh" ]]; then
  echo "找不到 ESP-IDF 6.0.2。请设置 P4DESK_IDF_PATH；参见 docs/build.md。" >&2
  exit 1
fi
if [[ -d /Volumes/work/esp ]]; then
  P4DESK_BUILD_DIR=${P4DESK_BUILD_DIR:-/Volumes/work/esp/build/p4desk}
else
  P4DESK_BUILD_DIR=${P4DESK_BUILD_DIR:-${TMPDIR:-/tmp}/p4desk-build}
fi
case "$P4DESK_BUILD_DIR" in
  *[!\ -~]*) echo "ESP-IDF 构建目录需要 ASCII 路径，请设置 P4DESK_BUILD_DIR。" >&2; exit 1;;
esac
if [[ -x /opt/homebrew/opt/python@3.13/bin/python3.13 ]]; then
  export PATH="/opt/homebrew/opt/python@3.13/bin:$PATH"
fi
source "$P4DESK_IDF_PATH/export.sh"
set -u
P4DESK_IDF_VERSION=$(idf.py --version)
if [[ "$P4DESK_IDF_VERSION" != "ESP-IDF v6.0.2" ]]; then
  echo "当前为 $P4DESK_IDF_VERSION，项目要求 ESP-IDF v6.0.2。" >&2
  exit 1
fi
rustup run nightly-2026-09-27 rustc --version
cd "$P4DESK_ROOT/firmware"
idf.py -B "$P4DESK_BUILD_DIR" "$@" build
echo "固件已生成：$P4DESK_BUILD_DIR/p4desk.bin"
