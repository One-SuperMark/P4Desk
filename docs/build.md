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

固件将 LCD 画面相对首版旋转 **180°**，Pad 与 USB 副屏统一生效。EK79007 使用硬件扫描方向，不增加每帧像素倒转或帧缓冲复制。

本地 EK79007 驱动的初始 MADCTL 为 `0x01`，`esp_lcd_panel_mirror(panel, false, true)` 将其设为 `0x02`，同时反转原始扫描的两个轴；具体寄存器定义见 [EK79007 数据手册 R36h](https://dl.espressif.com/dl/schematics/display_driver_chip_EK79007AD_datasheet.pdf)。该调用位于面板初始化后、背光点亮与 display owner 启动前，失败会明确返回。

GT911 保留物理原始坐标，在 `touch_task` 对每个触点先限制到有效像素范围，再转换 `x = 1023 - x`、`y = 599 - y`，随后提供给 Pad、原始多点触摸帧和 USB 回传。Mac 侧按 1024×600 逻辑坐标处理输入和画面。

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

脚本构建 arm64 SwiftPM release、Rust 字形工具，打包 Noto Regular 字体和 OFL 许可后签名。默认 ad-hoc 签名用于本机自用；已有签名身份可通过 `P4DESK_CODESIGN_IDENTITY` 指定。输出 `dist/P4Desk.app`。

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
