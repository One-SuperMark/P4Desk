# 原创桌面 SVG 图标

八个桌面图标由 P4Desk 项目自制，采用渐变圆角底板和白色几何符号，使用 MIT 许可。`source/*.svg` 是正式可编辑源。其 128×128 坐标按实际桌面、状态栏尺寸转换为矢量路径，由 tiny_gfx 做覆盖率抗锯齿后直接画入设备 RGB565 帧缓冲。

| ID | 图案 |
| --- | --- |
| clock | 钟面与指针 |
| timer | 秒表 |
| notes | 纸张与文本线条 |
| calculator | 显示框与按键 |
| mac | 普通桌面显示器 |
| settings | 齿轮 |
| display | 双显示器 |
| screen | 显示器电源 |

```sh
python3 scripts/generate-vector-icons.py
python3 scripts/generate-vector-icons.py --check
```

编译器同时处理本目录八个 SVG 和 `assets/ui_icons/source` 的 26 个通用 SVG，输出 `crates/tiny-flutter/src/graphics/svg_icons_generated.rs` 与 `assets/vector-icons.json`。源 SHA256 与生成几何 SHA256 可复现。固件存储几何，不存储这些图标的 PNG／RGB565／alpha 图集；旧八组 511,584 字节像素资源已移除。PNG 文件仅保留为素材预览，不链接进固件。

原 `AppIconAsset` 查询接口现在返回 `VectorIcon`，`DESKTOP_ICON_SIDE = 146` 继续作为桌面布局上限。详见 [矢量图标绘制](../../docs/vector-icons.md)。许可全文见 [LICENSE](LICENSE)。
