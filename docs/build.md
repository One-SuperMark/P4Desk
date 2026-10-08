# 中文构建说明

## Pad 局部刷新接口

`PlatformBackend::flush_strided` 可同步借用 RGB565 帧缓冲的一个区域，按原行距直接复制，省去滚动期间的中间打包。默认后端返回不支持并回退原 `flush`；P4 使用 `p4desk_pad_blit_rgb565`，`pixel_count` 与 `stride` 均以 `u16` 像素数计数（P4 上 `size_t`／Rust `usize` 为 32 位）。C 端核对矩形、行距和切片长度，借用在返回前结束，写入仍受原 Pad 锁保护，LCD 仍由唯一 display owner 提交。此为进程内 HAL 调用，USB 协议 v1 编码未改变。

Pad 按实际变化像素累计三块 LCD 各自的更新范围，小更新局部复制，大过渡使用 PPA。输出基址与 600 行扫描保持完整；取消或从 JPEG 返回时全量恢复。PPA／JPEG 输出的 task M2C 同步由项目包装按 32 KiB 分块，运行期启用轻量堆边界保护。设计、边界和检查记录见 [全应用显示与内存检查](display-memory-performance.md)；可分别运行 `scripts/test-pad-damage.sh`、`scripts/test-display-cache-sync.sh` 做 ASan／UBSan 回归。

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

构建过程由 CMake 统一传递 SDK、sdkconfig、include、链接依赖与 C 编译器给 Rust 的 `esp-idf-sys`／`embuild`，使用 `-Zbuild-std=std,panic_abort` 生成静态库。全局 `RUSTFLAGS` 同时传入 `espidf_time64` 和 `espidf_picolibc`，确保标准库也使用 ESP-IDF 6.0.2 的时间类型与文件打开标志。C／Rust 检查 `time_t`、`timeval`、布尔值、整数 ABI，以及 `O_CREAT`、`O_TRUNC`、`O_APPEND`、`O_EXCL` 与生成 C 绑定的一致性；缺少参数时拒绝构建。

`sdkconfig.defaults` 默认选择 `CONFIG_COMPILER_OPTIMIZATION_PERF=y`，C、JPEG 驱动、LCD 驱动与旋转拷贝使用 `-O2`。已有 `firmware/sdkconfig` 会覆盖默认值；升级旧构建目录时，在 `menuconfig → Compiler options → Optimization Level` 选择性能优化，再重新构建。可从构建目录 `compile_commands.json` 核对实际参数，不能仅凭默认配置认定产物已启用 `-O2`。

主要输出：

```text
p4desk.bin
p4desk.elf
bootloader/bootloader.bin
partition_table/partition-table.bin
flasher_args.json
```

## 屏幕方向

当前实机反馈旧画面上下颠倒后，将安装方向整体翻转 180°。当前板级配置为 `P4DESK_DISPLAY_ROTATION_DEGREES=0`、`P4DESK_TOUCH_ROTATION_DEGREES=180`，定义在 `firmware/components/board_p4/include/board_p4.h`。早期台面测试使用的是显示 180°、触摸 0°；旧验收文档保留当时配置，不代表当前方向。

面板扫描固定为 MADCTL `0x01`。Pad 从 Rust 逻辑画布同步输出，由唯一 display owner 按当前角度使用 PPA 复制／旋转，提交拒绝时退回 CPU。Mac 从能力协商读取实际角度，按需先旋转可见 1024×600 像素再编码 JPEG；P4 避免重复旋转。旧主机仍走解码暂存与裁切路径。灰度 JPEG 转换成 RGB565 后也按 session 的实际方向处理，608 行解码填充不会进入画面。

项目内 `lcd_frame_observer` 组件固定 ESP-IDF 6.0.2 DPI 源码 SHA256，仅在构建目录生成扩展，不修改全局 SDK。三块 LCD 缓冲各预留608行（总增加48KiB），DMA／面板继续扫描600行；精确指针容量查询确认可写范围。显示 owner 依据真实 completed／next 指针及连续 counter 切换缓冲，准备下一帧可与扫描重叠；同缓冲重复扫描、模式变更和迟到事件不会提前释放 DMA 所有权。事件丢失、溢出或期限超时会停止重用。

GT911 在 `touch_task` 中先限制坐标到有效像素范围，再按当前 180° 校正转换为逻辑坐标，统一提供给 Pad、原始多点触摸帧和 USB 回传。触摸和面板原生轴向分别校准，不能直接用显示角度替代 GT911 校正值。Mac 侧按 1024×600 逻辑坐标处理输入和画面。

运行 `./scripts/test-display.sh` 检查生产代码的 RGB565／灰度方向、重复帧重建、行填充、完整 600 行和边界保护，使用 ASan／UBSan。

`./scripts/test-pad-touch.sh` 检查 Pad 事件 FIFO 的短点击、运动历史、环形回绕、溢出取消和模式重置，使用 ASan／UBSan；C／Rust 的 12 字节事件 ABI 在固件构建时检查。

GT911 使用项目内专用 I²C IO：每次调用传入最多 50 ms 等待参数，状态读取开启 150 ms 批次预算，后续操作按剩余时间收紧参数；不再传入无限等待。IDF 内部可能将该参数分别用于多个等待阶段，因此它不是严格的墙钟最大耗时。没有新帧（状态 bit7=0）时不发送清状态 ACK，避免读到空状态后新到达的松手帧被清除；有效触点帧和零触点松手帧仍正常 ACK。

包装补报上游驱动忽略的 ACK 错误。读取失败取消当前手势，不能将失败当作 UP 执行按钮；等待控制器的有效零触点帧后才恢复输入。`p4desk_touch_health` 每 30 秒输出读取、真实帧、DOWN／UP／CANCEL、队列消费与恢复状态计数，不记录坐标或触点 ID。`./scripts/test-touch-io.sh` 直接编译生产 IO 包装，覆盖无帧 ACK 竞态、错误传播、超时参数与设备释放；`test-pad-touch.sh` 另验证取消和真实松手后的再次点击。

2026-10-08 的点击无响应排查、修正、构建和实机证据见 [触摸输入验收](acceptance-touch-input.json)。读取诊断口时出现新启动，之前故障现场已丢失；修正针对源码确认的隐患，具体故障触发原因和触摸恢复仍需实机确认。

`./scripts/test-display-pipeline.sh` 检查三缓冲状态及模式切换。`python3 -m unittest discover -s firmware/components/lcd_frame_observer/tests -v` 检查固定 SDK 扩展、实际 DMA 回调、容量查询和扫描几何；可通过 `IDF_PATH` 指向本机6.0.2。缺 SDK 的局部测试会明确跳过，不视为固件构建成功。

### JPEG 颜色范围

`firmware/components/jpeg_full_range` 同样只在构建目录生成固定 ESP-IDF 6.0.2 的 `jpeg_decode.c` 副本。生成器校验原源 SHA256 和唯一锚点；SDK 漂移、DMA2D 通道约束不符或替源次数不是一时停止配置，不退回未校验源。原 Apache-2.0 头和源文件名保留，全局 SDK 不修改。

JFIF JPEG 使用完整范围 BT.601，而该 SDK 的内置 DMA2D BT.601 参数为视频有限范围。局部扩展只在 JPEG 事务已独占 RX0、尚未启动 DMA 的原配置位置覆写硬件转换系数；此时不是持有 DMA2D 的 spinlock。灰度、BT.709、RGB565 字节顺序和其他 DMA 客户端保持原路径，每个新事务由自己的正常配置重新设定寄存器。无需增加全屏颜色修正或像素拷贝。

```sh
python3 -m unittest discover -s firmware/components/jpeg_full_range/tests -v
```

矩阵与寄存器测试证明生成源和 JFIF 数学／布局一致，完整固件构建证明 SDK 编译链接；实板颜色观感单独记录，不能用数学测试代替面板验收。

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

烧录脚本针对本板 CH343 USB TO UART 调试口，使用 `esptool --after no-reset`，成功写入后留在 ROM 下载模式并释放串口。随后通过指定 ESP-IDF Python 环境重新打开串口：打开前预置 DTR／RTS 为未断言状态，等待 200 毫秒，保持 DTR 未断言并执行 100 毫秒 RTS 硬复位，释放后保持串口打开 500 毫秒再关闭。写入失败不会执行启动复位。这样避免在释放 EN 的同一时刻关闭串口；操作系统或 USB 驱动仍可能产生短暂控制线变化，刷后画面方向和位置需要单独实机确认，不能只凭脚本退出码判定。

这块板的旧 `storage` 挂载返回 `SPIFFS_ERR_NOT_A_FS`，旧／新 SPIFFS 关键参数相同。完整备份确认新增设置区全部为空白，因此保留旧区并单独建立 P4Desk 设置区。没有格式化旧 `storage` 或 TF。

读取启动日志：

```sh
python3 scripts/device-tool.py monitor \
  --port /dev/cu.YOUR_P4_SERIAL \
  --python /absolute/path/idf6.0_python_env/bin/python \
  --duration 60 --reset --output .cache/device/boot.txt
```

日志去除设备 MAC／串口标识。完整备份回滚可使用 esptool 的 `write-flash 0 backup.bin`；只在确需恢复时执行，备份文件与当前设备必须对应。

日志监控同样在打开串口前预置 DTR／RTS 未断言。省略 `--reset` 时不主动发送复位脉冲；提供 `--reset` 时保持 DTR 未断言，通过 RTS 执行正常启动硬复位。不要同时打开其他串口终端。

连接不稳定时可为刷写指定 `--baud 115200`。备份已完成的情况无需重复备份；刷写脚本每次都会重新核对完整备份的 SHA256。

## Mac 应用

```sh
./scripts/build-macos.sh
open dist/P4Desk.app
```

脚本构建 arm64 SwiftPM release、Rust 字形工具，打包 HarmonyOS Sans SC Regular 字体和原许可后签名。默认选择本机唯一有效的 Developer ID Application 证书，字体 helper 与主 App 都启用 Hardened Runtime 并添加安全时间戳；没有证书或有多张时明确报错。可通过 `P4DESK_CODESIGN_IDENTITY` 指定身份，只有显式设为 `-` 才使用 ad hoc。输出 `dist/P4Desk.app`，安装时放入 `/Applications`。

更新安装时先退出正在运行的 P4Desk，完整替换应用包，再执行 `codesign --verify --deep --strict /Applications/P4Desk.app` 校验。应用数据位于 Application Support 和 UserDefaults。系统界面字形生成、默认同步字体与许可说明见 [Pad 界面字体](pad-typeface.md)。

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

应用图标采用项目自制 SVG，离线编译为静态 Rust 矢量几何；设备按实际尺寸做覆盖率抗锯齿。修改 SVG 后，可用 Python 标准库及固定工具链 rustfmt 重新生成并校验：

```sh
python3 scripts/generate-vector-icons.py
python3 scripts/generate-vector-icons.py --check
```

检查圆角与缩放质量时，可用同一 Rust 桌面与固定演示时间输出预览，包含八个桌面图标与两个状态栏小图标：

```sh
cargo run -p app-launcher --features screenshots --example pad-icons-preview -- \
  artifacts/pad-icons.png
```

当前矢量填充、描边、裁剪与多尺寸绘制说明见 [SVG 矢量图标](vector-icons.md)。

```sh
cargo run -p app-launcher --features screenshots --bin p4desk-simulator -- \
  --headless --demo --screen all --output artifacts/pad-screenshots

cargo build --release -p p4desk-fontpack
target/release/p4desk-fontpack bake \
  --font assets/fonts/HarmonyOS_Sans_SC_Regular.ttf \
  --snapshot tests/fixtures/snapshot-v1.json \
  --output .cache/demo-font.p4f --sizes 18,22,28,36
target/release/p4desk-fontpack validate \
  --input .cache/demo-font.p4f --snapshot tests/fixtures/snapshot-v1.json
```

交互模拟器可添加 `--features simulator`；无屏环境使用 `screenshots`。演示数据与设备上的真实数据独立。

### 翻页时钟预览

使用同一 Rust 控件、动画状态与局部绘制路径生成固定演示时间的预览：

```sh
cargo run -p app-launcher --features screenshots --example flip-clock-preview -- \
  artifacts/flip-clock
```

输出 1024×600 的 `000.png` 至 `190.png` 以及最终落稳的 `clock.png`，按 20 ms 演示时间间隔采样。演示从 2026-09-29 20:06:58 UTC 开始，包含秒变化与分钟进位，画面使用 12 小时制；文件用于界面审阅，不是开发板截屏或实机帧率记录。

数字采用 DINish Heavy 的原生 800 字重，启用 `tnum`／`lnum` 选择源字体中的等宽齐线数字。图集固定为 11 个 144×208 alpha8 字形，随固件内嵌；正常构建无需再次栅格化字体。重新生成仅需要 macOS、Swift／CoreText 和 Python 标准库，不需要 Pillow：

```sh
python3 scripts/generate-clock-assets.py --font heavy
python3 scripts/generate-clock-assets.py --font heavy --check
```

检查模式在临时目录生成并逐字节比较图集、元数据和来源说明；不修改已保存资源。源字体、尺寸、校验值、实际等宽字形 ID 与光学居中边界见 [时钟数字图集](../assets/clock/README.md)。

动画由单调时间驱动，持续 640 ms，采用半页透视、正反面交接、软阴影与轻微落稳回弹，以 20 ms 门限提交变化卡片的局部 dirty，结束时补齐完整展开帧。离开时钟、进入副屏或关闭屏幕后取消动画，重新进入直接显示当前时间。局部 dirty 降低 Rust 绘制／提取开销；C 显示 owner 仍沿用按板级方向校准的整屏复制和 LCD 提交路径。实现、参考来源与验证边界见 [翻页时钟动画](flip-clock-animation.md)；重绘门限不代表实测设备帧率。

USB 首帧启动过渡的主机验证：

```sh
cargo test -p app-launcher --test app_launch_tests
cargo test -p rust_main
./scripts/test-display-transition.sh
./scripts/test-display-pipeline.sh
./scripts/test-display.sh
```

`display-transition` 使用 ASan／UBSan 校验首帧之前不计时、epoch 隔离与最终 LCD 完成条件；不替代开发板 JPEG／LCD 实机验收。


### 显示中断与 Flash 保存

Pad 自动保存会话时写入内部 SPIFFS，LCD 扫描仍需持续服务 DMA 完成中断。项目启用 `CONFIG_LCD_DSI_ISR_CACHE_SAFE=y`，由 SDK 选择 DW-GDMA ISR 安全与内部 RAM 对象配置。FreeRTOS 的 ISR 函数保持不放入 Flash。仅将 ISR 代码放入 IRAM 不等于使用可在 Flash 操作期间响应的中断分配标志。

`CONFIG_ESP_MM_CACHE_MSYNC_C2M_CHUNKED_OPS=y`、`CONFIG_ESP_MM_CACHE_MSYNC_C2M_CHUNKED_OPS_MAX_LEN=0x8000` 将 C2M 缓存写回分成 32 KiB 的短临界区。display owner 在同步结束前持有输入锁、不发布目标缓冲，ISR 不写这些缓冲。

已有 `firmware/sdkconfig` 覆盖 defaults 时，需在 ESP-IDF 6.0.2 中将上述三项同步启用后重新构建；从实际生成的 `config/sdkconfig.h` 检查选项，从 ELF 检查 LCD/DMA 回调及 FreeRTOS ISR 依赖位于内部指令 RAM。`p4desk_display_health` 每 30 秒记录帧边界间隔、现有 cache 写回耗时/错误及 PPA 耗时/错误，不记录画面。欠载计数为零不等同于所有 DSI 链路错误都已排除。

### 整屏横向错位的诊断与扫描保护

ESP-IDF 6.0.2 将 bridge 的 `fifo_underrun_discard_vcnt` 设置为水平像素数。ESP32-P4 TRM 的 Register 43.15 将它定义为按行计数抑制欠载中断的阈值；本屏水平 1024 像素、垂直总行 636，因此此前的零欠载统计存在漏报风险。项目本地 DPI 补丁将阈值设为 0，保持视频时序、帧大小、旋转和 DMA 所有权不变。

`p4desk_dsi_host` 日志来自唯一 display owner 每 50 ms 读取的 Host INT_ST0/1。按 TRM §43.4.2.4，这两个寄存器读后清零；其他任务只读内部 RAM 累计快照，禁止直接读取硬件状态。`dpi_overflow_polls`、`dpi_underflow_polls` 分别统计采样见到 bit 7、bit 19 的次数，多次硬件事件可能合并在一次采样中。

Host 的首次／最近时间是读到报告的采样时间，不是硬件故障发生的精确时刻。首次采样可能读到面板初始化期间累积的状态；应分别记录首次状态与后续是否新增，不能把启动标记当作运行期零错误，也不能按首次采样时间定位启动中的某一条命令。

板级初始化将 DSI 使用的 DW-GDMA memory master 1 读取优先级设为 15，保留写优先级和其他 master 设置。TRM §21.2.1.16 说明较高 QoS 值表示较高仲裁优先级。此项降低 LCD 与缓存／PPA 竞争时的风险，不改变帧率，也不保证能恢复已经失步的显示流。构建、刷机、压力操作和人工实屏反馈分别记录在用量监控验收文件中。
