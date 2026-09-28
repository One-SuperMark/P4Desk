# 第三方来源与许可

| 内容 | 固定来源／版本 | 许可与保留位置 |
| --- | --- | --- |
| tiny-flutter、tiny_gfx、应用桌面、计算器 | pomelos-on-sale/esp32-rust-ui，`0b75870835902cbf950505d1539dbb8f6e0a7197` | MIT，`third_party/licenses/esp32-rust-ui-MIT.txt` |
| 7B GPIO、DSI 供电、EK79007 时序、SDMMC 引脚初始化 | 微雪 BSP 3.0.1，提取到 `firmware/components/board_p4` | Apache-2.0，原 SPDX 与 `Espressif-Apache-2.0.txt` |
| EK79007 驱动 | espressif/esp_lcd_ek79007 2.0.2~1 | Apache-2.0，`EK79007-Apache-2.0.txt` |
| GT911 与触摸接口 | espressif/esp_lcd_touch_gt911 1.2.1、esp_lcd_touch 1.2.1 | Apache-2.0，各 `*-Apache-2.0.txt` |
| TinyUSB | espressif/tinyusb 0.21.0~2 | MIT，`TinyUSB-MIT.txt` |
| FatFs 局部 exFAT 覆盖 | ESP-IDF 6.0.2 的 components/fatfs | 各源文件的 FatFs／Espressif 声明；说明在 `P4DESK-OVERRIDE.md` |
| Rust CMake 混合构建 | esp-rs/esp-idf-template CMake 模板 | 生成模板 MIT-0，来源在 `firmware/THIRD-PARTY.md` |
| CGVirtualDisplay 私有声明 | Stengo/DeskPad | MIT，完整声明在 `desktop/macos/THIRD_PARTY_NOTICES.md`，随 app 分发 |
| Noto Sans SC Regular | notofonts/noto-cjk 的 SubsetOTF/SC/NotoSansSC-Regular.otf | SIL OFL 1.1，`NotoSans-OFL.txt` 和 `NotoSans-copyright.txt` |
| 原框架 Roboto、DejaVu 回退字体及对应 baked 字形 | 参考项目 assets/fonts | Apache-2.0／DejaVu 许可，`Roboto-Apache-2.0.txt`、`DejaVu-fonts.txt` |

ESP-IDF 驱动依赖由 manifest 的 `==` 版本及 `firmware/dependencies.lock` 中的 component hash 固定。Rust 依赖由根 `Cargo.lock` 固定。字体原文件的长度和 SHA256 记录于 `assets/fonts/SOURCES.json`。

项目附带的 Noto 字体是未改动的官方 Regular OTF。系统／便签 P4F1 字形包由该字体生成，随包保留原 OFL 与版权声明。生成字形包不改变字形来源许可。

图标由代码绘制简单几何形状，界面和演示数据由本项目编写。参考项目的 Apple 风格图像、壁纸和商业音乐没有作为本产品资源分发。

桌面的图标绘制组件、图标网格、横向分页、页码圆点与状态栏后台应用入口基于参考项目相应 Rust 模块适配。具体职责与首版重写的纠偏记录见 [Rust UI 移植说明](rust-ui-port.md)。自制图标的 RGB565／alpha8 资源可用 `python3 scripts/generate-desktop-assets.py` 重现，源码和生成资源使用本项目 MIT 许可。

项目新代码使用根 MIT 许可；第三方文件与派生板级代码继续遵循其原许可。
