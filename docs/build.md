# 中文构建说明

## 固定环境

| 项目 | 版本 |
| --- | --- |
| ESP-IDF | 6.0.2 |
| Rust | nightly-2026-09-27，包含 rust-src 与 rustfmt |
| 固件目标 | riscv32imafc-esp-espidf |
| C 工具链 | ESP-IDF 6.0.2 的 riscv32-esp-elf；通过 CMake 传给 Rust |
| 首个主机环境 | Apple Silicon、macOS 27.0、Xcode Command Line Tools |

`rust-toolchain.toml` 和 `Cargo.lock` 固定 Rust 环境与依赖；ESP-IDF 的组件 manifest 与 `firmware/dependencies.lock` 固定驱动／TinyUSB 版本和组件 hash。CMake 检查 IDF 必须为 6.0.2。项目内 FatFs 仅启用 exFAT，全局 SDK 保持原配置。

首次安装 Rust 工具链：

```sh
rustup toolchain install nightly-2026-09-27 --profile minimal --component rust-src --component rustfmt
```

ESP-IDF 安装后使用其 `tools/idf_tools.py install --targets esp32p4` 与 `install-python-env` 建立工具／Python 环境。macOS 可通过 Homebrew 的 Python 3.13 执行这两条 SDK 命令。项目脚本优先使用该 Python，避免误选系统旧 Python 环境。

## P4 固件

在项目根目录执行：

```sh
./scripts/build-firmware.sh
```

本机默认 SDK 是 `/Volumes/work/esp/esp-idf-v6.0.2`，构建输出为 `/Volumes/work/esp/build/p4desk`。其他机器可指定：

```sh
P4DESK_IDF_PATH=/absolute/path/esp-idf-v6.0.2 \
P4DESK_BUILD_DIR=/absolute/ascii/path/p4desk-build \
./scripts/build-firmware.sh
```

构建目录使用英文路径。ESP-IDF 6.0.2 的编译参数响应文件在本机中文输出路径下发生损坏；源码可以保留在中文目录中。

构建过程由 CMake 统一传递 SDK、sdkconfig、include、链接依赖与 C 编译器给 Rust 的 `esp-idf-sys`／`embuild`，使用 `-Zbuild-std=std,panic_abort` 生成静态库。`espidf_time64` 在两侧统一启用，C／Rust 都检查 `time_t`、`timeval`、布尔值和整数 ABI。

主要输出：

```text
p4desk.bin
p4desk.elf
bootloader/bootloader.bin
partition_table/partition-table.bin
flasher_args.json
```

## 屏幕方向

本次按当前摆放，对送往 LCD 的画面像素进行 **180° 软件旋转**，Pad 与 USB 副屏统一生效。用户实板反馈 EK79007 MADCTL 命令返回成功后，屏幕没有实际旋转，因此采用最终帧缓冲反向拷贝。

方向在 `firmware/components/board_p4/include/board_p4.h` 配置：`P4DESK_DISPLAY_ROTATION_DEGREES=180`、`P4DESK_TOUCH_ROTATION_DEGREES=0`。这与本机旧 `waveshare_touch_paint` 的实际编译配置一致：画面软件旋转 180°，GT911 使用原始坐标。面板扫描固定为 MADCTL `0x01`，软件旋转只作用于 display owner 拥有的 `FB_BUILDING` 目标。

Pad 从未旋转的 Rust 画布反向拷贝；JPEG 按实际 stride 读取，只将可见的 600 行反向写入，解码到 608 行时末尾填充不会进入画面。RGB565 按 16 位像素处理，灰度先转换成 RGB565。旋转合并在原有拷贝中，不分配额外帧缓冲，不增加第二次全帧反转。

GT911 保留本次已经调整后的原始坐标，`touch_task` 将每个触点限制到有效像素范围，统一提供给 Pad、原始多点触摸帧和 USB 回传。触摸和面板原生轴向分别校准，不能直接用显示角度替代 GT911 校正值。Mac 侧按 1024×600 逻辑坐标处理输入和画面。

运行 `./scripts/test-display.sh` 检查生产代码的 RGB565／灰度方向、重复帧重建、行填充、完整 600 行和边界保护，使用 ASan／UBSan。

## 首次刷写与恢复备份

先选择实际串口路径，再执行备份。不要在同一串口同时运行串口监视器。

```sh
python3 scripts/backup-device.py \
  --port /dev/cu.YOUR_P4_SERIAL \
  --python /absolute/path/idf6.0_python_env/bin/python \
  --output backups/first-flash/p4-before-p4desk.bin
```

脚本分段读取 32 MiB Flash，每段检查设备 MD5，合并后计算 SHA256，同时提取独立 NVS 恢复文件。串口噪声时重试并降低波特率；备份目录权限为 0700，备份不进入 Git。

校验完成后刷写：

```sh
python3 scripts/device-tool.py flash \
  --port /dev/cu.YOUR_P4_SERIAL \
  --python /absolute/path/idf6.0_python_env/bin/python \
  --backup backups/first-flash/p4-before-p4desk.bin \
  --build-dir /absolute/ascii/path/p4desk-build
```

只写 `flasher_args.json` 列出的 bootloader、分区表和应用。NVS 地址／大小保持为 0x9000／0x6000；旧 `storage` 区保留 0x810000 起的 7 MiB。P4Desk 的设置／删除记录改用独立 `p4settings` SPIFFS，0xf10000 起的 1 MiB。首次初始化仅允许全 0xFF 的空白新分区；既有数据挂载失败时保持原状。TF 卡不参与刷写。

这块板的旧 `storage` 挂载返回 `SPIFFS_ERR_NOT_A_FS`，旧／新 SPIFFS 关键参数相同。完整备份确认新增设置区全部为空白，因此保留旧区并单独建立 P4Desk 设置区。没有格式化旧 `storage` 或 TF。

读取启动日志：

```sh
python3 scripts/device-tool.py monitor \
  --port /dev/cu.YOUR_P4_SERIAL \
  --python /absolute/path/idf6.0_python_env/bin/python \
  --duration 60 --reset --output .cache/device/boot.txt
```

日志去除设备 MAC／串口标识。完整备份回滚可使用 esptool 的 `write-flash 0 backup.bin`；只在确需恢复时执行，备份文件与当前设备必须对应。

连接不稳定时可为刷写指定 `--baud 115200`。备份已完成的情况无需重复备份；刷写脚本每次都会重新核对完整备份的 SHA256。

## Mac 应用

```sh
./scripts/build-macos.sh
open dist/P4Desk.app
```

脚本构建 arm64 SwiftPM release、Rust 字形工具，打包 Noto Regular 字体和 OFL 许可后签名。默认选择本机唯一有效的 Developer ID Application 证书，字体 helper 与主 App 都启用 Hardened Runtime 并添加安全时间戳；没有证书或有多张时明确报错。可通过 `P4DESK_CODESIGN_IDENTITY` 指定身份，只有显式设为 `-` 才使用 ad hoc。输出 `dist/P4Desk.app`，安装时放入 `/Applications`。

首次进入副屏时授予屏幕录制权限；需要触摸和快捷键时授予辅助功能权限。配置窗口显示当前权限和连接状态，并提供系统设置入口。便签／快捷面板编辑和同步不要求录屏权限。

独立探测：

```sh
./scripts/probe-macos.sh
```

探测使用合成画面检查 JPEG 编码，创建 1024×600 虚拟屏后检查系统登记并销毁。实际 USB 呈现与触摸需要已刷入本项目固件的开发板。

## 制作交付包

完成上述构建和记录、提交源码后运行：

```sh
python3 scripts/package-release.py --build-dir /Volumes/work/esp/build/p4desk
```

输出 `dist/P4Desk-0.1.0/` 与 `dist/P4Desk-0.1.0.zip`，包含 Mac App、可刷写固件及 ELF、字形工具和字体、说明／许可证、对应 Git 提交的完整源码归档，以及逐文件 SHA256 清单。脚本校验 App 签名、必需资源和源码工作区状态；备份、缓存和用户数据不进入交付包。

解压后的固件位于 `P4Desk-0.1.0/firmware`，可直接作为 `device-tool.py flash --build-dir` 参数。完整源码在 `P4Desk-0.1.0-source.tar.gz`；要重新构建先解压源码归档，在其根目录运行构建脚本。

## Rust 桌面预览与字形工具

应用图标采用项目自制的 128×128 RGB565／alpha8 资源，已经保存在源码中。修改图标几何定义后，可用 Python 标准库重新生成：

```sh
python3 scripts/generate-desktop-assets.py
```

```sh
cargo run -p app-launcher --features screenshots --bin p4desk-simulator -- \
  --headless --demo --screen all --output artifacts/pad-screenshots

cargo build --release -p p4desk-fontpack
target/release/p4desk-fontpack bake \
  --font assets/fonts/NotoSansSC-Regular.otf \
  --snapshot tests/fixtures/snapshot-v1.json \
  --output .cache/demo-font.p4f --sizes 18,22,28,36
target/release/p4desk-fontpack validate \
  --input .cache/demo-font.p4f --snapshot tests/fixtures/snapshot-v1.json
```

交互模拟器可添加 `--features simulator`；无屏环境使用 `screenshots`。演示数据与设备上的真实数据独立。
