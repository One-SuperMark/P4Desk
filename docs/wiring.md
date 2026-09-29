# 7B 接线与启动

以开发板丝印和[微雪官方硬件说明](https://docs.waveshare.com/ESP32-P4-WIFI6-Touch-LCD-7B)核对端口。

## 三个 USB 口

| 板上端口 | 用途 | 本项目用途 |
| --- | --- | --- |
| Type-A，USB-OTG | USB 2.0 High Speed | 副屏视频、便签／字形同步、动作与多点触摸回传 |
| Type-C，USB1.1 FS Direct Out | P4 的 USB Serial/JTAG，支持供电／烧录／调试 | 可用于供电和调试；不承担副屏 HS 数据 |
| Type-C，USB TO UART | 串口桥，支持供电／烧录／调试 | 首次芯片识别、完整 Flash 备份、刷写和启动日志 |

开发联调可同时保留串口线与 HS 数据线。确认 Type-A USB-OTG 数据线是微雪示例指定的连接方式，并核对其 VBUS／供电连接；不要用端口外形推断主机／设备角色。开发板电源开关必须打开。

本项目固件将 Type-A 的 P4 OTG 配置为 **USB device / HS**，Mac 作为 host。枚举为 `P4Desk`，开发 VID/PID 为 `303A:4044`，vendor interface 0（bulk OUT 0x01／IN 0x81），HID Consumer interface 1。这个 VID/PID 用于当前个人开发固件；量产分发需使用自己的设备标识。

板上状态栏的“USB 未连接”指这一路 **USB-OTG 数据接口尚未枚举**。仅连接 Type-C 时，开发板可以供电、运行 Pad，并通过相应接口刷写／调试；Mac 配套应用仍会显示未连接。USB Serial/JTAG 调试接口可能显示为 `303A:1001`，与本项目的数据接口 `303A:4044` 分属不同 USB 控制器。

连接 OTG 数据接口后，Mac 应用自动握手并校时。打开应用、编辑便签和同步不需要屏幕录制或辅助功能权限；开启副屏与执行触摸／快捷键输入时再处理对应权限。

微雪随板清单包含双 USB-A 公头线。对于 USB-C 接口的 Mac，数据连接可按以下路径接入；已有带 USB-A 母口的数据扩展坞时，也可从其 USB-A 数据口接出：

```text
Mac USB-C → USB-C 公／USB-A 母的数据转接头 → 随板双 USB-A 公头线 → 板上 Type-A USB-OTG
```

线材来源见[微雪官方随板清单](https://www.waveshare.com/product/arduino/displays/esp32-p4-wifi6-touch-lcd-7b.htm)，Mac 转接方式见 [Apple USB-C 转 USB 转换器说明](https://support.apple.com/zh-cn/111751)。板端供电按官方电源要求设置，具体实板与线材的 VBUS／供电组合验收见验收记录。

## TF 卡与屏幕

1. 断电时将现有 TF 卡插入板载卡槽，避免带电拔插造成正在写入的资源代次中断。
2. 本项目启动会检测实际 FAT 类型、容量、空闲空间与读写状态。FAT32 和 exFAT 通过项目内 FatFs 配置支持。
3. DSI 屏幕排线与触摸排线按微雪标识接好。初始化使用 EK79007 的 1024×600 RGB565 时序，画面按当前摆放校正；GT911 的所有触点使用同一方向配置，保留触点 ID。方向实现见 [构建说明](build.md#屏幕方向)。
4. 首次启动先核对 RGB 色块、底部 24 行、四角触摸和屏幕方向，再验收副屏输入。代码构建通过不代替这些物理检查。

TF 挂载／读写异常时，界面保留基础时钟、计算器和计时工具，显示存储状态。项目不会自动格式化 TF 卡。设置与删除记录写入独立 Flash 日志槽，资源和便签正文写入 TF。

## 模式切换

- 默认开机进入 Pad，未校时显示“待校时”。
- 打开 Mac 菜单栏应用后自动握手和 USB 校时。
- Mac 菜单或 Pad 设置中的“进入 USB 副屏”触发显示器创建、捕获验证与设备模式协商。
- Mac 菜单退出或板上三指长按一秒退出。退出后应在板上看到完整 Pad 界面。
- 拔掉 HS 数据线、Mac 应用退出／休眠或设备心跳超时，清空旧帧并释放输入，返回 Pad。
- Mac 的“显示器”设置可将 P4 Desk 排列在主屏的右、左或上方。触摸按实时全局坐标换算，包含负坐标。

## 常见连接状态

| 状态 | 检查 |
| --- | --- |
| Type-C 已插上，板上仍显示 USB 未连接 | 将 Mac 数据连接接到 Type-A USB-OTG 大接口；Type-C 供电／调试连接不承担本项目的同步和副屏协议 |
| 有串口、Mac 应用没有设备 | 检查 Type-A HS 数据口是否连接，以及是否已刷入本项目固件；Mac 应识别 `303A:4044`，USB Serial/JTAG 的 `303A:1001` 不是配套应用的数据接口 |
| USB 配件访问被拒绝 | 在 macOS 配件权限提示／系统设置允许设备，再点重新连接 |
| 便签不能同步 | 检查 TF 就绪状态、空间、字形生成器与字体资源 |
| 副屏进入失败 | 查看 Mac 提示中的录屏权限、虚拟屏登记、编码或设备呈现回执状态 |
| 已连接，但电脑窗口更新慢 | 在 App“副屏”页检查实际 USB 速率。Full Speed 为 12 Mbps，High Speed 为 480 Mbps；端口支持 HS 不代表本次连接已达到 HS。使用上面的官方数据连接路径，重连后核对速率；若仍为 Full Speed，继续核对线材、转接头及 Hub 路径 |
| 副屏可见、触摸无作用 | 授予 P4 Desk 辅助功能权限，检查当前屏幕排列 |
| 手势退出后仍显示旧画面 | 检查主机退出流程与 Pad 完整重绘的实机验收记录 |

资料：[微雪硬件说明](https://docs.waveshare.com/ESP32-P4-WIFI6-Touch-LCD-7B)、[USB 示例源码](https://github.com/waveshareteam/ESP32-P4-WIFI6-Touch-LCD-7B/tree/main/examples/esp-idf/12_usb_extend_screen)。

副屏传输为 1024×600 的完整 JPEG 帧。Full Speed 的物理线速仅约 1.5 MB/s，实际有效吞吐还要扣除协议开销。例如每帧 100 KiB 时，即使忽略开销，传输上限也只有约 14.6 FPS。应在持续拖动／滚动或移动测试窗口时观察有效 FPS；静态画面不会持续产生新帧，静态低 FPS 不表示面板扫描频率降低。仅提高采集 FPS 无法增加 USB 链路带宽。
