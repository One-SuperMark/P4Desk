# P4Desk · Colloid 双主题图标

按与 Numix 套装相同的 **25 个用途、ID 和分组**，整理出浅色／深色各 25 枚，共 **50 份独立主题 SVG**。保留 25 份未修改上游原图，整包共 75 个 SVG。保留 Colloid 的圆角方形造型。

## 使用入口

| 内容 | 路径 |
| --- | --- |
| 浅色资源 | `themes/light/svg/<分组>/<ID>.svg` |
| 深色资源 | `themes/dark/svg/<分组>/<ID>.svg` |
| 主题切换、用途筛选、尺寸检查、下载 | [预览.html](预览.html) |
| 全部 25 对图标 | [主题对照.png](主题对照.png) |
| 单主题总览 | [浅色](图标总览-浅色.png) / [深色](图标总览-深色.png) |
| 140／68／28 px 对照 | [主题尺寸对照.png](主题尺寸对照.png) |
| ID 与两套实际路径 | [主题用途对照.csv](主题用途对照.csv) |

`manifest.json` 中的 `icons[].variants.light.file`、`icons[].variants.dark.file` 为可使用资源路径。`file` 指向保留原图，仅用于来源追溯。两套文件名和 ID 一致，切换主题只需切换目录。

## 适配规则

- 浅色采用浅底板、明确的主色符号；深色降低底板亮度并提亮主体。同一用途在两套主题中保持色系和轮廓一致。
- 上游安装脚本将多数 Dark 应用图标链接到 Light。本套是基于上游矢量的 **P4Desk 自定义双主题衍生版**，并非冒称官方两套独立应用配色。
- 逐图指定颜色映射，保留文件夹、纸张、金属、按键的层次；不使用 CSS 反相或页面滤镜。
- 21 份上游原图含共 23 个内嵌 PNG 图层，均已核对为透明单色阴影。主题副本移除这些位图以及 SVG 模糊滤镜；保留矢量主体，柔化矢量阴影。上游原图按原字节保留。
- USB 副屏、显示器、亮度、绿色计时器备选这 4 枚使用上游符号，补上 `56×56 / rx=13` 的 Colloid 比例圆角底板，并为小尺寸轻微加粗；其余 21 枚沿用上游完整应用或设备图形。
- SUB2API Monitor 采用系统监测曲线，增强曲线线宽和网格对比；补强 USB 标志和深色日历数字，改善小尺寸显示。计时器原图中无关的音乐符号残留与画布外矩形设为隐藏。
- 两套均为透明画布，统一 `width="64" height="64" viewBox="0 0 64 64"`。原始路径通过外层变换归一化，局部路径、几何尺寸及原变换不改写；新增底板、描边单独记录。
- 在 140／68／28 px 槽位按完整画布等比绘制；圆角底板可见宽度约为槽位的 87.5%。无需使用 Numix 的 128／62／26 px 缩小绘制规则。

## 25 个用途

| 分组 | 名称 | 浅色 SVG | 深色 SVG | 上游源文件 |
| --- | --- | --- | --- | --- |
| 现有桌面 | 时钟 | [clock](themes/light/svg/desktop/clock.svg) | [clock](themes/dark/svg/desktop/clock.svg) | `src/apps/scalable/preferences-system-time.svg` |
| 现有桌面 | 番茄钟 | [timer](themes/light/svg/desktop/timer.svg) | [timer](themes/dark/svg/desktop/timer.svg) | `src/apps/scalable/pomidor.svg` |
| 现有桌面 | 便签 | [notes](themes/light/svg/desktop/notes.svg) | [notes](themes/dark/svg/desktop/notes.svg) | `src/apps/scalable/basket.svg` |
| 现有桌面 | 计算器 | [calculator](themes/light/svg/desktop/calculator.svg) | [calculator](themes/dark/svg/desktop/calculator.svg) | `src/apps/scalable/calc.svg` |
| 现有桌面 | Mac 控制 | [mac](themes/light/svg/desktop/mac.svg) | [mac](themes/dark/svg/desktop/mac.svg) | `src/apps/scalable/preferences-desktop-keyboard.svg` |
| 现有桌面 | 设置 | [settings](themes/light/svg/desktop/settings.svg) | [settings](themes/dark/svg/desktop/settings.svg) | `src/apps/scalable/preferences-system.svg` |
| 现有桌面 | USB 副屏 | [display](themes/light/svg/desktop/display.svg) | [display](themes/dark/svg/desktop/display.svg) | `src/actions/24/org.remmina.Remmina-multi-monitor-symbolic.svg` |
| 现有桌面 | 关闭屏幕 | [screen](themes/light/svg/desktop/screen.svg) | [screen](themes/dark/svg/desktop/screen.svg) | `src/apps/scalable/shutdown.svg` |
| 规划应用 | 文件管理 | [file-manager](themes/light/svg/planned/file-manager.svg) | [file-manager](themes/dark/svg/planned/file-manager.svg) | `src/apps/scalable/file-manager.svg` |
| 规划应用 | Office 文档 | [office-viewer](themes/light/svg/planned/office-viewer.svg) | [office-viewer](themes/dark/svg/planned/office-viewer.svg) | `src/apps/scalable/gnome-documents.svg` |
| 规划应用 | 用量监控 | [sub2api-monitor](themes/light/svg/planned/sub2api-monitor.svg) | [sub2api-monitor](themes/dark/svg/planned/sub2api-monitor.svg) | `src/apps/scalable/utilities-system-monitor.svg` |
| 设置与设备 | Wi-Fi | [wifi](themes/light/svg/settings/wifi.svg) | [wifi](themes/dark/svg/settings/wifi.svg) | `src/devices/scalable/network-wireless.svg` |
| 设置与设备 | 蓝牙 | [bluetooth](themes/light/svg/settings/bluetooth.svg) | [bluetooth](themes/dark/svg/settings/bluetooth.svg) | `src/apps/scalable/bluetooth.svg` |
| 设置与设备 | 显示器 | [display-settings](themes/light/svg/settings/display-settings.svg) | [display-settings](themes/dark/svg/settings/display-settings.svg) | `src/devices/symbolic/video-display-symbolic.svg` |
| 设置与设备 | 日期与时间 | [date-time](themes/light/svg/settings/date-time.svg) | [date-time](themes/dark/svg/settings/date-time.svg) | `src/apps/scalable/calendar.svg` |
| 设置与设备 | 存储 | [storage](themes/light/svg/settings/storage.svg) | [storage](themes/dark/svg/settings/storage.svg) | `src/apps/scalable/org.gnome.DiskUtility.svg` |
| 设置与设备 | 关于 | [about](themes/light/svg/settings/about.svg) | [about](themes/dark/svg/settings/about.svg) | `src/apps/scalable/userinfo.svg` |
| 设置与设备 | USB | [usb](themes/light/svg/settings/usb.svg) | [usb](themes/dark/svg/settings/usb.svg) | `src/apps/scalable/usb-creator.svg` |
| 设置与设备 | 电池 | [battery](themes/light/svg/settings/battery.svg) | [battery](themes/dark/svg/settings/battery.svg) | `src/apps/scalable/gnome-power-manager.svg` |
| 设置与设备 | 亮度 | [brightness](themes/light/svg/settings/brightness.svg) | [brightness](themes/dark/svg/settings/brightness.svg) | `src/status/32/video-display-brightness.svg` |
| 设置与设备 | 音量与媒体 | [audio](themes/light/svg/settings/audio.svg) | [audio](themes/dark/svg/settings/audio.svg) | `src/devices/scalable/audio-speakers.svg` |
| 备选 | 时钟备选 | [clock-alarm](themes/light/svg/alternatives/clock-alarm.svg) | [clock-alarm](themes/dark/svg/alternatives/clock-alarm.svg) | `src/apps/scalable/org.gnome.Evolution-alarm-notify.svg` |
| 备选 | 计时器备选 | [timer-green](themes/light/svg/alternatives/timer-green.svg) | [timer-green](themes/dark/svg/alternatives/timer-green.svg) | `src/actions/24/chronometer.svg` |
| 备选 | 便签备选 | [notes-paper](themes/light/svg/alternatives/notes-paper.svg) | [notes-paper](themes/dark/svg/alternatives/notes-paper.svg) | `src/apps/scalable/com.github.philip_scott.notes-up.svg` |
| 备选 | 快捷键备选 | [mac-shortcuts](themes/light/svg/alternatives/mac-shortcuts.svg) | [mac-shortcuts](themes/dark/svg/alternatives/mac-shortcuts.svg) | `src/apps/scalable/preferences-desktop-keyboard-shortcuts.svg` |

## 三个新增应用与接入边界

- **文件管理**：蓝色文件夹，作为 TF 卡目录浏览、文件查看与管理入口。
- **Office 文档**：文档与阅读眼镜，作为 Office 文档统一查看入口。支持格式由后续功能实现决定。
- **SUB2API Monitor**：监测曲线，用于 Token、账号额度、费用和服务状态入口。

时钟仍带固定指针，实时钟面需先拆除固定指针并标定动态指针。日期中的 **26**、电池填充均是静态装饰；实时 Wi-Fi 分级、USB／TF 状态和电量仍应由业务状态驱动。28 px 预览主要用于入口辨识，文档细线、日历数字、钟面刻度等不保证逐个清晰可读。

本次完成图标资源整理，未修改 USB 副屏固件、Rust UI 或 Mac 应用。主题 SVG 已无位图、CSS 样式块或模糊滤镜，但仍包含路径、渐变和变换，接入固件几何编译链前仍需离线展开并检查支持子集。预览由 librsvg 渲染，未进行固件构建、刷机或 RGB565 实机验收。

## 来源与许可

上游：[vinceliuice/Colloid-icon-theme](https://github.com/vinceliuice/Colloid-icon-theme)。固定提交：`ceac6608ecd0e40025cbc2ebbd32bf0e0f4ebc6a`。下载日期：2026-09-30。

保留 [上游 README](UPSTREAM-README.md)、[GPL v3 原文](LICENSE)、[来源与修改说明](SOURCE-README.txt)。上游声明 GPL-3.0；衍生主题沿用 GPL-3.0，并在每个 SVG 的 `desc`、清单与本说明中标明修改和日期。未运行上游安装脚本，未安装到系统图标目录。

## 校验与重建

[manifest.json](manifest.json) 记录原图、主题、调色配方的 SHA256；[校验记录.json](校验记录.json) 记录 50 个资源 × 3 种尺寸的 150 次渲染检查、透明角与浅深像素差异。[原图位图层核对.json](原图位图层核对.json) 保留被清理图层的像素核对结果。

```sh
python3 scripts/curate.py
node scripts/render-preview.cjs
node scripts/audit-source-images.cjs
python3 scripts/package.py
```

全部 Node 脚本需能解析 `sharp`，或设置 `P4DESK_SHARP_MODULE` 为已有模块绝对路径。运行 `curate.py` 可从包内保留原图直接重建；邻接的 `../upstream/source` 存在时会核对固定原图哈希。`scripts/theme-palettes.json` 保存逐图调色、归一化和线宽配方。

生成流程会验证原始矢量几何保留、主题无位图或外部依赖、颜色映射完整、SVG 引用完整、下载链接存在与 ZIP 完整性。几何验证不表示模糊阴影保持不变；清理项及新增边框已在各图 `adaptation` 中单独记录。
