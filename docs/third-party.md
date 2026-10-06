# 第三方来源与许可

| 内容 | 固定来源／版本 | 许可与保留位置 |
| --- | --- | --- |
| Folio 视觉设计、色板与分组控件规则 | [McCal-Codes/folio](https://github.com/McCal-Codes/folio)，`2590bc164aaccaca79dc445272be6c56dff07dc1` | MIT，保留 `third_party/folio/LICENSE`；Rust 适配说明见 `docs/folio-ui.md` |
| Colloid 深浅图标与静态矢量派生 | 用户提供的 P4Desk 精选包；上游 `vinceliuice/Colloid-icon-theme`，`ceac6608ecd0e40025cbc2ebbd32bf0e0f4ebc6a` | GPL-3.0，`third_party/colloid-p4desk/{dark,light}/LICENSE`；源 SHA256 与规范化记录见 `assets/colloid-icons.json` |
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
| 翻页时钟、用量监控主数使用的 DINish Heavy／Black 及 alpha8 数字图集 | 官方 [playbeing/dinish](https://github.com/playbeing/dinish)，固定 `a5f3b2a3b932336225815bf9005e3b72cc3de71c`，静态 4.007 / Heavy 800、Black 900 | SIL OFL 1.1，`third_party/licenses/DINish-OFL.txt` 保留原版权声明；固定源地址及 SHA256 见 `assets/fonts/SOURCES.json` |
| 先前时钟及样式对比使用的 Roboto Mono Bold | Google 官方 [googlefonts/RobotoMono](https://github.com/googlefonts/RobotoMono)，固定 `895ec691990d041dd727c7b5afa3ce56525d98e6`，静态 3.001 / Bold 700 | SIL OFL 1.1，`third_party/licenses/RobotoMono-OFL.txt` 保留原版权声明 |

ESP-IDF 驱动依赖由 manifest 的 `==` 版本及 `firmware/dependencies.lock` 中的 component hash 固定。Rust 依赖由根 `Cargo.lock` 固定。字体原文件的长度和 SHA256 记录于 `assets/fonts/SOURCES.json`。

当前系统／默认便签 P4F1 字形由未修改的官方 HarmonyOS Sans SC Regular TTF 光栅化，普通拉丁回退使用同源 HarmonyOS Sans Regular。未修改原字体轮廓、未制作修改版 TTF；完整中文 TTF 随 P4Desk Mac 应用保留原版权和许可分发，交付工具目录不再单独复制字体。协议中的 alpha8 字形属于应用使用原字体的渲染资源。系统默认字体和可选择的便签／按钮字体分别解析，详见 [Pad 界面字体](pad-typeface.md)。

项目仍保留先前未改动的官方 Noto Regular OTF及其 OFL／版权，用于历史资源与对比。字体不统一改标为根 MIT 许可。

翻页时钟使用未修改的官方 DINish Heavy 静态 TTF，通过字体内置 `tnum`／`lnum` 选择真正等宽的齐线数字，保留原字体无斜线的 `0`。这些选中字形的 hmtx advance 均为 524 字体单位；字母和默认比例数字没有被修改为等宽。`assets/clock/flip-digits.alpha` 从原轮廓生成，图集固定槽位并将每个字形光学居中到翻页中轴，派生资源延续 OFL 1.1。重新生成由 Python 标准库调用 macOS Swift／CoreText 辅助脚本，正常 Rust 构建直接内嵌图集。分发时保留源字体版权及许可。

图标由代码绘制简单几何形状，界面和演示数据由本项目编写。参考项目的 Apple 风格图像、壁纸和商业音乐没有作为本产品资源分发。

桌面的图标绘制组件、图标网格、横向分页、页码圆点与状态栏后台应用入口基于参考项目相应 Rust 模块适配。具体职责与首版重写的纠偏记录见 [Rust UI 移植说明](rust-ui-port.md)。自制 SVG 图标及静态 Rust 矢量几何可用 `python3 scripts/generate-vector-icons.py` 重现，源码和生成几何使用本项目 MIT 许可；通用图标的许可同时保存在 `assets/ui_icons/LICENSE`。

项目新代码使用根 MIT 许可；第三方文件与派生板级代码继续遵循其原许可。

Folio 分支的状态电池数字同样使用固定 DINish Heavy 原始轮廓。`scripts/generate-battery-vectors.swift` 从本地固定 SHA256 字体提取 11 个等宽数字／横线，生成 `apps/app-launcher/src/battery_digits_generated.rs`；这些字形遵循已有 SIL OFL 1.1 及 `third_party/licenses/DINish-OFL.txt`，不使用 Apple 字体。

用量监控的主数内嵌同一份未经修改的 DINish Heavy TTF，使用有界 Latin 字形缓存绘制。数字居中到统一槽位、逗号保留自然宽度；这只是排版，不修改字体轮廓或源字体的字符 advance。系统中文继续使用 HarmonyOS Sans。

## Numix Circle 深浅图标（2026-09-30）

用户提供 `/Volumes/work/workspace/Numix-Circle-SVG/P4Desk精选/` 内的深浅衍生 SVG，来源清单记录上游 `numixproject/numix-icon-theme-circle` 的 `6d4a4aad60994e688ab9dc8f021cb4ec86e9e532`。集成每套 21 枚：8 个现有桌面图标、3 个新增入口和 10 个设置图标；4 个备选图标未选用。

这些 SVG 及其规范化 SVG、生成的 `numix_icons_generated.rs` 静态几何使用 **GPL-3.0-or-later**，保留完整许可证、来源清单、配色表与修改说明于 `third_party/numix-p4desk/` 和 `assets/numix/LICENSE`。它们不改标为根 MIT 许可。通用自制操作 SVG、原有软件组件和字体继续分别按各自许可证标注。

构建阶段用固定 fontTools 4.60.1 解析路径、弧线和变换，生成器属于项目工具代码；转换后仍为矢量。时钟去除固定指针与指针阴影，由真实时间绘制指针。可识别的圆形贝塞尔路径转换为现有圆形抗锯齿快路径，识别误差上限为 128 单位图标画布上的 0.16 单位。详细资源哈希与转换输出见 `assets/numix-icons.json`。

## WhiteSur 双主题图标（2026-09-30）

从用户提供的 `/Volumes/work/workspace/WhiteSur-SVG/P4Desk精选/` 导入深浅各 21 个实际使用的 SVG。清单记录上游 `vinceliuice/WhiteSur-icon-theme` 固定提交 `73d8040da51a9ed74e47c7366e7e9ff437601a5c`，GPL-3.0。原字节主题 SVG、作者、源清单与许可保存在 `third_party/whitesur-p4desk/`；文件 SHA256 在生成时验证。

`assets/whitesur/` 保留规范化派生矢量，完整 64×64 画布等比转为 128×128，保留多段、非垂直与径向渐变、透明度、圆角轮廓。经过数量校验后删除 4 层时钟固定指针／阴影，设备动态绘制当前时间。源包已移除内嵌位图阴影；固件未引入位图替代。源包的 USB 矩形裁剪经包围盒验证冗余后展开。生成器会拒绝新增的未支持元素／非冗余裁剪。源清单中的 4 个备选项未导入。

新增 Folio 浅色底板为项目原有原创 SVG 的衍生，仍使用 MIT，不混入第三方图标许可。两者生成记录分别见 `assets/whitesur-icons.json`、`assets/folio-light-icons.json`。
