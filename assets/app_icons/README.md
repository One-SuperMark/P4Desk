# 原创桌面 SVG 图标

八个桌面图标由 P4Desk 项目自制，采用 Folio 风格的圆形底板、柔和配色、实心符号和浅色叠层，使用 MIT 许可。`source/*.svg` 是正式可编辑源。外圆保持直径 112 个 SVG 坐标单位，内部图形（含描边和叠层）的最长边统一为 67.2，即外圆直径的 **60%**，等比缩放而不拉伸。桌面 140 px 画布上分别对应 122.5 px 与 73.5 px；Dock、信息卡片和启动动画共用这套源。其 128×128 坐标按实际桌面、状态栏尺寸转换为矢量路径，由 tiny_gfx 做覆盖率抗锯齿后直接画入设备 RGB565 帧缓冲。

| ID | 图案 |
| --- | --- |
| clock | 静态 SVG 钟面 + 随本地时间更新的时、分、秒针 |
| timer | 秒表 |
| notes | 纸张与文本线条 |
| calculator | 显示框与按键 |
| mac | 叠层 Command 键帽 |
| settings | 齿轮 |
| display | 双显示器 |
| screen | 电源环 |

```sh
python3 scripts/generate-vector-icons.py
python3 scripts/generate-vector-icons.py --check
```

编译器同时处理本目录八个 SVG 和 `assets/ui_icons/source` 的 34 个通用 SVG，输出 `crates/tiny-flutter/src/graphics/svg_icons_generated.rs` 与 `assets/vector-icons.json`。源 SHA256 与生成几何 SHA256 可复现。固件存储几何，不存储这些图标的 PNG／RGB565／alpha 图集；旧八组 511,584 字节像素资源已移除。PNG 文件仅保留为素材预览，不链接进固件。

原 `AppIconAsset` 查询接口现在返回 `VectorIcon`，`DESKTOP_ICON_SIDE = 146` 继续作为桌面布局上限。本分支新增样式说明见 [Folio 移植](../../docs/folio-ui.md)。详见 [矢量图标绘制](../../docs/vector-icons.md)。许可全文见 [LICENSE](LICENSE)。

时钟的固定指针已从 `clock.svg` 移除，由 `live_clock_icon.rs` 按与首页数字相同的本地时间绘制圆头抗锯齿矢量指针。卡片、桌面、最近应用 Dock 共用此路径；启动动画保留点击时的指针快照并按原节奏淡出。尚未校时时仅显示钟面。内圆比例与外圆尺寸保持不变。

## 可切换的桌面图标

此目录现在是“设置 → 外观 → 图标主题 → Folio”的原创圆形图标，新增文件管理、Office、用量监控三个图形，仍使用 MIT 许可。Numix Circle 与 Colloid 使用各自目录中的深浅配色及许可证。三套主题均保留，切换与资源说明见 `docs/icon-themes.md`。
