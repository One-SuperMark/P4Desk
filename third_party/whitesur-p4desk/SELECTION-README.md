# P4Desk · WhiteSur 双主题图标

与 Numix、Colloid 套装沿用相同的 **25 个用途、ID 和分组**：8 枚桌面、3 枚规划应用、10 枚设置／设备、4 枚备选。浅色、深色各 25 枚，共 **50 份独立 SVG**；另外保留 25 份原始上游 SVG，整包共 75 个 SVG。

## 使用入口

| 内容 | 路径 |
| --- | --- |
| 浅色主题 | `themes/light/svg/<分组>/<ID>.svg` |
| 深色主题 | `themes/dark/svg/<分组>/<ID>.svg` |
| 交互预览、筛选、尺寸切换与下载 | [预览.html](预览.html) |
| 25 对图标总览 | [主题对照.png](主题对照.png) |
| 分主题总览 | [浅色](图标总览-浅色.png) / [深色](图标总览-深色.png) |
| 桌面／Dock／小图对照 | [主题尺寸对照.png](主题尺寸对照.png) |
| 用途与路径对照 | [主题用途对照.csv](主题用途对照.csv) |

应用接入时使用 `manifest.json` 的 `icons[].variants.light.file` 或 `icons[].variants.dark.file`；两套相同 ID 和文件名。`file` 字段指向保留原图，供来源核对。

## WhiteSur 适配规则

- 保留 WhiteSur 的 macOS 风格、圆角轮廓、矢量渐变和立体层次。浅色尽量延续原生观感；深色调整底板、纸张、金属与主符号的颜色和亮度。
- 上游 Light/Dark 共用主要 scalable 应用素材。本套是 **P4Desk 自定义双主题衍生版**，并非官方两套独立应用配色。
- 逐图设定颜色映射及必要的部件覆写；不使用整体反相或 CSS 滤镜。保留局部矢量路径、形状尺寸和原变换，外层归一化到透明 `64×64` 画布。
- 19 份上游 SVG 含 24 个内嵌 PNG：23 个黑色阴影、1 个文件夹蓝色发光层。已核对并从主题副本移除，保留文件夹矢量主体；模糊滤镜同时清理，矢量渐变继续保留。原始 SVG 按原字节保存。
- 文件管理选用上游 `alternative` 蓝色文件夹，显示器选用 `alternative` 显示器造型；便于对应 TF 卡文件和显示器设置。
- USB 副屏、亮度、绿色计时器备选这 3 枚使用上游符号，音量与快捷键备选使用上游独立设备图形，共 5 枚补上 WhiteSur 风格渐变圆角底板。
- 增强监测曲线及 USB 标记；计时器中与用途无关的音乐符号残留和画布外矩形设为隐藏。细边界、部件覆写、归一化变换逐项写入 `adaptation`。
- 设置原图有一处缺失渐变引用，主题版将该无效填充显式设为 `none`；修复前后的 140 px librsvg 渲染像素一致，原图仍按原字节保留。
- 140／68／28 px 槽位按完整画布等比绘制；底板可见宽度约占画布 87.5%。小尺寸主要检查入口辨识，金属细节、刻度和文档细线不保证逐个可读。

## 全部资源

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
| 规划应用 | 文件管理 | [file-manager](themes/light/svg/planned/file-manager.svg) | [file-manager](themes/dark/svg/planned/file-manager.svg) | `alternative/apps/scalable/file-manager.svg` |
| 规划应用 | Office 文档 | [office-viewer](themes/light/svg/planned/office-viewer.svg) | [office-viewer](themes/dark/svg/planned/office-viewer.svg) | `src/apps/scalable/gnome-documents.svg` |
| 规划应用 | 用量监控 | [sub2api-monitor](themes/light/svg/planned/sub2api-monitor.svg) | [sub2api-monitor](themes/dark/svg/planned/sub2api-monitor.svg) | `src/apps/scalable/utilities-system-monitor.svg` |
| 设置与设备 | Wi-Fi | [wifi](themes/light/svg/settings/wifi.svg) | [wifi](themes/dark/svg/settings/wifi.svg) | `src/devices/scalable/network-wireless.svg` |
| 设置与设备 | 蓝牙 | [bluetooth](themes/light/svg/settings/bluetooth.svg) | [bluetooth](themes/dark/svg/settings/bluetooth.svg) | `src/apps/scalable/bluetooth.svg` |
| 设置与设备 | 显示器 | [display-settings](themes/light/svg/settings/display-settings.svg) | [display-settings](themes/dark/svg/settings/display-settings.svg) | `alternative/apps/scalable/preferences-desktop-display.svg` |
| 设置与设备 | 日期与时间 | [date-time](themes/light/svg/settings/date-time.svg) | [date-time](themes/dark/svg/settings/date-time.svg) | `src/apps/scalable/calendar.svg` |
| 设置与设备 | 存储 | [storage](themes/light/svg/settings/storage.svg) | [storage](themes/dark/svg/settings/storage.svg) | `src/apps/scalable/gnome-disks.svg` |
| 设置与设备 | 关于 | [about](themes/light/svg/settings/about.svg) | [about](themes/dark/svg/settings/about.svg) | `src/apps/scalable/userinfo.svg` |
| 设置与设备 | USB | [usb](themes/light/svg/settings/usb.svg) | [usb](themes/dark/svg/settings/usb.svg) | `src/apps/scalable/usb-creator.svg` |
| 设置与设备 | 电池 | [battery](themes/light/svg/settings/battery.svg) | [battery](themes/dark/svg/settings/battery.svg) | `src/apps/scalable/gnome-power-manager.svg` |
| 设置与设备 | 亮度 | [brightness](themes/light/svg/settings/brightness.svg) | [brightness](themes/dark/svg/settings/brightness.svg) | `src/status/32/video-display-brightness.svg` |
| 设置与设备 | 音量与媒体 | [audio](themes/light/svg/settings/audio.svg) | [audio](themes/dark/svg/settings/audio.svg) | `src/devices/scalable/audio-speakers.svg` |
| 备选 | 时钟备选 | [clock-alarm](themes/light/svg/alternatives/clock-alarm.svg) | [clock-alarm](themes/dark/svg/alternatives/clock-alarm.svg) | `src/apps/scalable/org.gnome.Evolution-alarm-notify.svg` |
| 备选 | 计时器备选 | [timer-green](themes/light/svg/alternatives/timer-green.svg) | [timer-green](themes/dark/svg/alternatives/timer-green.svg) | `src/actions/24/chronometer.svg` |
| 备选 | 便签备选 | [notes-paper](themes/light/svg/alternatives/notes-paper.svg) | [notes-paper](themes/dark/svg/alternatives/notes-paper.svg) | `src/apps/scalable/com.github.philip_scott.notes-up.svg` |
| 备选 | 快捷键备选 | [mac-shortcuts](themes/light/svg/alternatives/mac-shortcuts.svg) | [mac-shortcuts](themes/dark/svg/alternatives/mac-shortcuts.svg) | `src/devices/scalable/input-keyboard.svg` |

## 三个新增应用

- **文件管理**：蓝色文件夹，用于 TF 卡目录浏览、文件查看和管理。
- **Office 文档**：文档与阅读眼镜，作为统一查看入口。具体支持格式由后续功能实现决定。
- **SUB2API Monitor**：绿色监测屏与曲线，作为 Token、账号额度、费用和服务状态入口。

时钟固定指针、日历数字 **26** 和电池绿色填充仍是静态图形。实时钟面需拆除固定指针并标定动态指针；Wi-Fi 分级、USB／TF 在线状态和电量需由业务驱动。

本次整理独立素材包，未修改 USB 副屏工程。主题 SVG 已无位图、CSS 样式块或模糊滤镜，但仍包含渐变、路径、裁切和变换；固件接入前需离线展开和支持子集检查。本次预览由 sharp / librsvg 渲染，未进行固件构建、刷机或 RGB565 实机验收。

## 来源与许可

上游：[vinceliuice/WhiteSur-icon-theme](https://github.com/vinceliuice/WhiteSur-icon-theme)，固定提交 `73d8040da51a9ed74e47c7366e7e9ff437601a5c`，获取日期 2026-09-30。

保留 [上游 README](UPSTREAM-README.md)、[作者信息](UPSTREAM-AUTHORS.txt)、[GPL v3 原文](LICENSE) 与 [来源及修改说明](SOURCE-README.txt)。上游声明 GPL-3.0；衍生主题沿用 GPL-3.0，每个 SVG 的 `desc` 标明修改日期、固定来源和修改范围。未运行上游安装脚本。

## 校验与重建

[manifest.json](manifest.json) 记录原图、主题和配方 SHA256；[校验记录.json](校验记录.json) 记录 50 个主题资源在 140／68／28 px 的 150 次渲染、透明角和浅深像素差异。[原图位图层核对.json](原图位图层核对.json) 记录被移除的阴影／蓝色光晕的来源与像素证据。

```sh
python3 scripts/curate.py
node scripts/render-preview.cjs
node scripts/audit-source-images.cjs
python3 scripts/package.py
```

Node 脚本依赖已有 `sharp`，可使用 `P4DESK_SHARP_MODULE` 指定模块绝对路径。`curate.py` 能从包内 `svg/` 和 manifest 重建；邻接 `../upstream/source` 存在时会读取并核对固定原图。调色、部件覆写与外层变换均见 [theme-palettes.json](scripts/theme-palettes.json)。

打包验证主题无位图或外部资源、SVG 引用完整、下载链接存在、渲染记录与当前哈希一致、ZIP 完整性。矢量几何保留不代表模糊与发光效果不变，清理项在清单中单独记录。
