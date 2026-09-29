# P4 Desk macOS 配套应用

SwiftUI 菜单栏应用，使用原生 `IOUSBHost` 与 P4 Desk 固件通信。当前验证主机为 **macOS 27.0（26A428）、Apple Silicon arm64**。SwiftPM 的最低部署声明为 macOS 14；其他 macOS 版本与 Intel 的实际运行尚未验收。

## 打开已构建交付包

解压 `P4Desk-0.1.0.zip` 后，进入其中的 `P4Desk-0.1.0/` 目录运行：

```sh
open ./P4Desk.app
```

App 已包含字体和字形生成器，使用时无需构建工具链。交付目录中的 `scripts/` 提供备份与刷写工具，构建和测试脚本位于另附的源码归档中。

## 从源码构建与打开

在源码项目根目录运行以下命令。使用交付 ZIP 时，先解压其中的 `P4Desk-0.1.0-source.tar.gz`，进入 `P4Desk-0.1.0-source/`：

```sh
./scripts/build-macos.sh
open dist/P4Desk.app
```

脚本会构建 Swift release 应用；字体 helper 缺失时自动运行 `cargo build --release -p p4desk-fontpack`。打包必须包含 Regular OTF、字体 helper、OFL、字体版权声明与来源清单，缺失时构建失败。应用 bundle ID 固定为 `com.p4desk.mac`。

默认自动选择本机唯一有效的 `Developer ID Application` 证书，同时签署字体 helper 与主 App，并添加安全时间戳。没有有效证书或存在多张时，脚本报错；可设置 `P4DESK_CODESIGN_IDENTITY` 为证书 SHA1 或完整名称显式选择。仅本机开发时，显式设置 `P4DESK_CODESIGN_IDENTITY=-` 才使用 ad hoc 签名。

在首次授权前确定签名身份，并在后续版本沿用；ad hoc 重建或更换签名身份后系统可能要求重新授权。字体 helper 的开发覆盖路径可通过 `P4DESK_FONTPACK_BIN` 指定。

## 使用

1. 用数据线连接开发板 **USB HS 数据口，即板上 Type-A USB-OTG 大接口**。Type-C 小接口用于供电／烧录调试，不能传输此副屏协议；只连接 Type-C 时，App 会显示 USB HS 未连接。
2. 启动或再次打开 App 会显示便签与按钮配置窗口。关闭窗口后 App 继续在菜单栏运行，可从菜单栏“打开便签与按钮配置”重新打开。内容自动保存在本机 Application Support/P4Desk/state.json；编辑后点“同步更改”。
3. 同步先读取设备状态，离线删除记录优先合并。中文字库与 Snapshot 使用 `sync_begin → RESOURCE/ACK → sync_commit` 完整一代提交。失败时设备保留上一有效代次。
4. 按钮支持快捷键、启动应用、播放/音量等媒体键。快捷键和应用动作按设备已提交的按钮代次查找；媒体操作由板上的标准 USB HID Consumer 接口执行。
5. 开启副屏需要屏幕录制权限；触摸指针与快捷键需要辅助功能权限。启动应用、编辑与同步不会请求这些权限。权限可在应用设置中手动管理。
6. 副屏启用后，可在系统显示设置中排列 P4 Desk，并拖入普通窗口。单指点击/拖动、双指滚动；板上三指长按退出。断线、退出与睡眠会释放指针、停止捕获并回到 Pad。

首帧成功编码、设备 `set_mode` ACK 与实际 `frame_presented` 回执都通过后，应用才显示副屏已开启。没有设备或接口打不开时显示真实断开/错误状态。

## 实现边界

- 私有 `CGVirtualDisplay` 的声明集中在 `P4DeskNative/VirtualDisplay.mm`，由 DeskPad MIT 声明适配；许可证随应用保留。该路线用于个人和小范围直接分发，不能作为 Mac App Store 路线。
- 显示器固定 1024×600、1 倍比例、单个 60 Hz 模式；采集与编码目标为 60 FPS。ScreenCaptureKit 使用原生回调频率（`minimumFrameInterval=.zero`），再由两帧容量的 gate 限流；诊断 JSON 记录实际配置。实际呈现 FPS 取决于画面细节、USB、JPEG、固件解码与 LCD，以设备回执统计为准。用户已接受当前速度，后续优先改善画质。
- 描述符使用稳定身份与 sRGB 色度坐标、自有串行队列。当前系统会缓存已停用显示器的身份；不要在失败时循环生成随机身份，也不要把 `applySettings == true` 当成系统登记成功。
- 当前 CoreGraphics 进程可能缓存 `CGDisplayMode`。创建之后用活动显示列表与尺寸校验，probe 还用独立子进程校验模式的逻辑/像素尺寸。
- `ScreenCaptureKit` 只匹配新显示器的 ID，显式使用 sRGB。编码优先 ImageIO 最高质量 baseline JPEG：本机验证质量 1 输出 4:4:4 完整色度和 JFIF 全范围 BT.601。此前 VideoToolbox JPEG 各质量均输出 4:2:0，实际 YCbCr 系数符合 BT.709 且码流未标记矩阵，因此不再默认使用该路径。发送前验证 1024×600、8 bit SOF0、完整 SOI/EOI、合法采样和 1 MiB 上限；仅对超大帧有界重新编码，记录每帧实际质量、采样与尝试次数。
- 新固件宣告主机旋转能力后，Mac 用 GPU 将可见像素旋转 180° 再编码，P4 直接解码到 LCD 缓冲；旧固件继续接收原方向 JPEG 并在板上旋转。GPU 池有界，编码、交给主界面与 USB 发送均保留在途帧和最新待处理帧；停止后拒绝迟到结果。
- USB 仅匹配 VID `303A` / PID `4044`、Vendor interface 0，使用 OUT `01` / IN `81`；HID interface 1 留给系统。发送保持一帧在途和最新一帧缓存，控制消息优先于下一完整帧。
- USB 扫描使用 `IOServiceMatching("IOUSBHostInterface")` 加 `IOPropertyMatch` 的精确属性约束。本机 macOS 27 的只读 probe 已验证：旧 helper 平铺字典匹配 0 个，标准属性字典匹配 1 个 Vendor 接口。接口或端点打不开时显示一次包含阶段和错误码的提示，不把失败隐藏成“未连接”。
- 每秒心跳。接口重开前保留至少 3.25 秒 OUT 静默，允许设备清除已中断的部分帧；静默结束后才报告连接并开始 HELLO 超时计时。

## 验证与诊断

以下命令也在源码项目根目录执行：

```sh
swift test --package-path desktop/macos
./scripts/probe-macos.sh
```

probe 运行协议自检、合成图 JPEG 编码、真实虚拟显示器创建/登记/销毁。**不会请求录屏或辅助功能权限，也不会采集用户屏幕内容。** 虚拟屏验收必须满足 online、active、1024×600 logical/pixel 和销毁后 removed。

应用“副屏”页可导出性能 JSON，仅包含单调时间戳与统计值。近 30 秒 P95 按实际收到 LCD 呈现回执的帧计算，保留采集、编码起止、入队、USB 发送完成和回执时间；有效 FPS 按实际呈现回执计算。回执耗时包含 USB 回传，**LCD 光学延迟仍需独立实测**。没有真实 capture/设备呈现时没有性能样本。

捕获时间源逐帧记录：优先可验证的 WindowServer 显示时间，再尝试 sample PTS，最后使用捕获回调的主机单调时间。编辑窗口关闭时继续采集并记录元数据，仅减少界面统计刷新；捕获持有允许系统休眠的活动租约，退出、停止或启动失败均释放。保存窗口出现前已冻结导出样本，避免窗口阻塞改变被记录的 FPS。

副屏页显示接口所属设备的实际协商速率；Full Speed 为 12 Mbps，High Speed 为 480 Mbps。性能 JSON 新增 JPEG 字节数、实际 OUT 传输耗时、发送前等待、编码器硬件状态／参数返回值、编码待处理帧替换计数，以及固件回传的解码／裁剪旋转／LCD 等待耗时。有效 FPS 每秒刷新并纳入空闲时间；ScreenCaptureKit 静态画面不产生新帧，低更新率不代表 LCD 扫描频率降低。

对比版本时，开启副屏后在源码根目录运行固定移动测试窗口：

```sh
mkdir -p .cache
xcrun swiftc -O -parse-as-library scripts/performance-pattern.swift -o .cache/performance-pattern
.cache/performance-pattern .cache/performance-pattern.json 60
```

测试程序使用预构建字体及明确的窗口生命周期，需编译后运行。它在已存在的 P4 Desk 显示器上按 60 次／秒更新，显示 45 秒自制文本和移动色块，结束自动关闭。可追加参数 `30` 使用旧版 30 次／秒场景。约第 33 秒导出 App 性能 JSON，使用最近 30 秒样本；核对 `capture_target_fps`、模式、USB 实际速率、编码质量和测试窗口的 `placement_matches_target=true`，再比较两次结果。该窗口不捕获或保存用户画面，元数据记录更新／绘制次数及单调起止时间。此测试场景不能代替不同办公画面、输入及长期稳定性的验收。

清晰度／颜色的自制测试窗口包含不同字号中文、红橙黄绿蓝紫色块、橙色渐变、灰阶、单像素彩线和底部 24 行。开启副屏后在项目根目录执行，默认显示五分钟，之后自动关闭：

```sh
xcrun swiftc -O -parse-as-library scripts/quality-pattern.swift -o /tmp/p4desk-quality-pattern
/tmp/p4desk-quality-pattern 300
```

它只创建测试窗口，不读取屏幕内容，也不直接访问 USB。最高质量 JPEG 仍是有损图像，面板原生分辨率仍为 1024×600、RGB565；不能把编码质量 1 等同于无损或 Retina 显示。

实测 probe 证据位于 [acceptance/mac27-arm64-probes.json](acceptance/mac27-arm64-probes.json)。源码测试与本机 probe 不等同于板卡副屏、拖窗、触摸、快捷键、睡眠和持续运行的完整硬件验收。

## 参考

- [DeskPad](https://github.com/Stengo/DeskPad)：私有 CGVirtualDisplay 声明与原实现，MIT。
- [go-macos/virtualdisplay](https://github.com/go-macos/virtualdisplay/blob/main/virtualdisplay_darwin.go)：独立研究并实测 CoreGraphics 缓存与显示器生命周期，BSD-3-Clause；用于核对系统行为。
- [Apple IOUSBHostInterface](https://developer.apple.com/documentation/iousbhost/iousbhostinterface)：原生用户空间 USB interface/pipe。
- [Apple ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit)：显示器捕获。
