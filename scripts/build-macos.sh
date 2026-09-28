#!/bin/zsh
set -euo pipefail
TASK_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASK_PACKAGE="$TASK_ROOT/desktop/macos"
TASK_APP="$TASK_ROOT/dist/P4Desk.app"
TASK_FONT="$TASK_ROOT/assets/fonts/NotoSansSC-Regular.otf"
TASK_OFL="$TASK_ROOT/third_party/licenses/NotoSans-OFL.txt"
TASK_COPYRIGHT="$TASK_ROOT/third_party/licenses/NotoSans-copyright.txt"
TASK_FONT_SOURCES="$TASK_ROOT/assets/fonts/SOURCES.json"
TASK_FONTPACK="${P4DESK_FONTPACK_BIN:-$TASK_ROOT/target/release/p4desk-fontpack}"
TASK_CODESIGN_IDENTITY="${P4DESK_CODESIGN_IDENTITY:-}"
if [[ -z "$TASK_CODESIGN_IDENTITY" ]]; then
  TASK_DEVELOPER_IDS=()
  while IFS= read -r TASK_DEVELOPER_ID; do
    if [[ -n "$TASK_DEVELOPER_ID" ]]; then
      TASK_DEVELOPER_IDS+=("$TASK_DEVELOPER_ID")
    fi
  done < <(security find-identity -v -p codesigning | awk '$2 ~ /^[0-9A-Fa-f]+$/ && length($2) == 40 && /"Developer ID Application:/ { print $2 }')
  if (( ${#TASK_DEVELOPER_IDS[@]} == 0 )); then
    print -u2 "未找到有效的 Developer ID Application 签名证书。请安装证书或设置 P4DESK_CODESIGN_IDENTITY；仅本机开发时可显式设置为 - 使用 ad hoc。"
    exit 1
  elif (( ${#TASK_DEVELOPER_IDS[@]} != 1 )); then
    print -u2 "存在多张有效的 Developer ID Application 证书。请设置 P4DESK_CODESIGN_IDENTITY 指定证书的 SHA1 或完整名称。"
    exit 1
  fi
  TASK_CODESIGN_IDENTITY="${TASK_DEVELOPER_IDS[1]}"
fi
TASK_CODESIGN_FLAGS=(--force --sign "$TASK_CODESIGN_IDENTITY" --options runtime)
if [[ "$TASK_CODESIGN_IDENTITY" != "-" ]]; then
  TASK_CODESIGN_FLAGS+=(--timestamp)
fi
if [[ ! -x "$TASK_FONTPACK" ]]; then
  cargo build --manifest-path "$TASK_ROOT/Cargo.toml" --release -p p4desk-fontpack
fi
for TASK_RESOURCE in "$TASK_FONT" "$TASK_OFL" "$TASK_COPYRIGHT" "$TASK_FONT_SOURCES" "$TASK_FONTPACK"; do
  if [[ ! -s "$TASK_RESOURCE" ]]; then
    print -u2 "打包必需资源缺失或为空：$TASK_RESOURCE"
    exit 1
  fi
done
if [[ ! -x "$TASK_FONTPACK" ]]; then
  print -u2 "字体同步工具不可执行：$TASK_FONTPACK"
  exit 1
fi
swift build -c release --package-path "$TASK_PACKAGE" --product P4Desk --arch arm64
TASK_BIN_DIR="$(swift build -c release --package-path "$TASK_PACKAGE" --arch arm64 --show-bin-path)"
mkdir -p "$TASK_APP/Contents/MacOS" "$TASK_APP/Contents/Resources"
cp "$TASK_BIN_DIR/P4Desk" "$TASK_APP/Contents/MacOS/P4Desk"
cp "$TASK_PACKAGE/Info.plist" "$TASK_APP/Contents/Info.plist"
cp "$TASK_PACKAGE/THIRD_PARTY_NOTICES.md" "$TASK_APP/Contents/Resources/THIRD_PARTY_NOTICES.md"
cp "$TASK_FONT" "$TASK_APP/Contents/Resources/NotoSansSC-Regular.otf"
rm -f "$TASK_APP/Contents/Resources/NotoSansSC-VF.ttf"
cp "$TASK_OFL" "$TASK_APP/Contents/Resources/NotoSans-OFL.txt"
cp "$TASK_COPYRIGHT" "$TASK_APP/Contents/Resources/NotoSans-copyright.txt"
cp "$TASK_FONT_SOURCES" "$TASK_APP/Contents/Resources/FONT-SOURCES.json"
cp "$TASK_FONTPACK" "$TASK_APP/Contents/Resources/p4desk-fontpack"
codesign "${TASK_CODESIGN_FLAGS[@]}" --identifier com.p4desk.fontpack "$TASK_APP/Contents/Resources/p4desk-fontpack"
codesign "${TASK_CODESIGN_FLAGS[@]}" --identifier com.p4desk.mac "$TASK_APP"
codesign --verify --deep --strict "$TASK_APP"
print "Mac 应用已构建：$TASK_APP"
