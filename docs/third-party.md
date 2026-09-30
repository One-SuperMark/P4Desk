# 第三方来源与许可

| 内容 | 固定来源／版本 | 许可与保留位置 |
| --- | --- | --- |
| tiny-flutter、tiny_gfx、应用桌面、计算器 | pomelos-on-sale/esp32-rust-ui，`0b75870835902cbf950505d1539dbb8f6e0a7197` | MIT，`third_party/licenses/esp32-rust-ui-MIT.txt` |
| 7B GPIO、DSI 供电、EK79007 时序、SDMMC 引脚初始化 | 微雪 BSP 3.0.1，提取到 `firmware/components/board_p4` | Apache-2.0，原 SPDX 与 `Espressif-Apache-2.0.txt` |
| EK79007 驱动 | espressif/esp_lcd_ek79007 2.0.2~1 | Apache-2.0，`EK79007-Apache-2.0.txt` |
| GT911 与触摸接口 | espressif/esp_lcd_touch_gt911 1.2.1、esp_lcd_touch 1.2.1 | Apache-2.0，各 `*-Apache-2.0.txt` |
| ESP-Hosted SDIO Wi-Fi / BLE、Wi-Fi Remote | espressif/esp_hosted 1.4.7、esp_wifi_remote 1.2.5，按微雪 Wi-Fi 示例锁定 | Apache-2.0，组件原许可证与 `third_party/licenses/Espressif-Apache-2.0.txt`；局部兼容副本见 `firmware/compat` |
| TinyUSB | espressif/tinyusb 0.21.0~2 | MIT，`TinyUSB-MIT.txt` |
| FatFs 局部 exFAT 覆盖 | ESP-IDF 6.0.2 的 components/fatfs | 各源文件的 FatFs／Espressif 声明；说明在 `P4DESK-OVERRIDE.md` |
| Rust CMake 混合构建 | esp-rs/esp-idf-template CMake 模板 | 生成模板 MIT-0，来源在 `firmware/THIRD-PARTY.md` |
| CGVirtualDisplay 私有声明 | Stengo/DeskPad | MIT，完整声明在 `desktop/macos/THIRD_PARTY_NOTICES.md`，随 app 分发 |
| 系统界面与默认同步字体 HarmonyOS Sans SC Regular／HarmonyOS Sans Regular | 官方 OpenHarmony `utils_system_resources`，固定 `4d96c1c7158103732b687e9ea47f4454f6df3ea4`，原静态 1.9 / 400 | HarmonyOS Sans Fonts License Agreement，`third_party/licenses/HarmonyOS-Sans-LICENSE.txt`；固件设置页和 Mac 字库页标注字体名称 |
| 先前字形与对比使用的 Noto Sans SC Regular | notofonts/noto-cjk 的 SubsetOTF/SC/NotoSansSC-Regular.otf | SIL OFL 1.1，`NotoSans-OFL.txt` 和 `NotoSans-copyright.txt` |
| 原框架 Roboto、DejaVu 回退字体及对应 baked 字形 | 参考项目 assets/fonts | Apache-2.0／DejaVu 许可，`Roboto-Apache-2.0.txt`、`DejaVu-fonts.txt` |
| 翻页时钟 DINish Heavy／Black 及 alpha8 数字图集 | 官方 [playbeing/dinish](https://github.com/playbeing/dinish)，固定 `a5f3b2a3b932336225815bf9005e3b72cc3de71c`，静态 4.007 / Heavy 800、Black 900 | SIL OFL 1.1，`third_party/licenses/DINish-OFL.txt` 保留原版权声明；固定源地址及 SHA256 见 `assets/fonts/SOURCES.json` |
| 先前时钟及样式对比使用的 Roboto Mono Bold | Google 官方 [googlefonts/RobotoMono](https://github.com/googlefonts/RobotoMono)，固定 `895ec691990d041dd727c7b5afa3ce56525d98e6`，静态 3.001 / Bold 700 | SIL OFL 1.1，`third_party/licenses/RobotoMono-OFL.txt` 保留原版权声明 |

ESP-IDF 驱动依赖由 manifest 的 `==` 版本及 `firmware/dependencies.lock` 中的 component hash 固定。Rust 依赖由根 `Cargo.lock` 固定。字体原文件的长度和 SHA256 记录于 `assets/fonts/SOURCES.json`。

当前系统／默认便签 P4F1 字形由未修改的官方 HarmonyOS Sans SC Regular TTF 光栅化，普通拉丁回退使用同源 HarmonyOS Sans Regular。未修改原字体轮廓、未制作修改版 TTF；完整中文 TTF 随 P4Desk Mac 应用保留原版权和许可分发，交付工具目录不再单独复制字体。协议中的 alpha8 字形属于应用使用原字体的渲染资源。系统默认字体和可选择的便签／按钮字体分别解析，详见 [Pad 界面字体](pad-typeface.md)。

项目仍保留先前未改动的官方 Noto Regular OTF及其 OFL／版权，用于历史资源与对比。字体不统一改标为根 MIT 许可。

翻页时钟使用未修改的官方 DINish Heavy 静态 TTF，通过字体内置 `tnum`／`lnum` 选择真正等宽的齐线数字，保留原字体无斜线的 `0`。这些选中字形的 hmtx advance 均为 524 字体单位；字母和默认比例数字没有被修改为等宽。`assets/clock/flip-digits.alpha` 从原轮廓生成，图集固定槽位并将每个字形光学居中到翻页中轴，派生资源延续 OFL 1.1。重新生成由 Python 标准库调用 macOS Swift／CoreText 辅助脚本，正常 Rust 构建直接内嵌图集。分发时保留源字体版权及许可。

图标由代码绘制简单几何形状，界面和演示数据由本项目编写。参考项目的 Apple 风格图像、壁纸和商业音乐没有作为本产品资源分发。

桌面的图标绘制组件、图标网格、横向分页、页码圆点与状态栏后台应用入口基于参考项目相应 Rust 模块适配。具体职责与首版重写的纠偏记录见 [Rust UI 移植说明](rust-ui-port.md)。自制 SVG 图标及静态 Rust 矢量几何可用 `python3 scripts/generate-vector-icons.py` 重现，源码和生成几何使用本项目 MIT 许可；通用图标的许可同时保存在 `assets/ui_icons/LICENSE`。

项目新代码使用根 MIT 许可；第三方文件与派生板级代码继续遵循其原许可。
