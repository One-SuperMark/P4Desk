# Folio 分支原创曲线壁纸

P4Desk 原创几何曲线与柔和灰蓝配色，MIT 许可。未使用 Folio 的截图壁纸。

`folio-waves.spans` 是 `scripts/generate-folio-wallpaper.py` 从四条三次贝塞尔曲线生成的抗锯齿水平填充记录。每条记录为 8 字节：小端 y/x/长度（各 u16），alpha 和图层索引（各 u8）。生成器使用四个横向采样和纵向覆盖率，保留 1024×600 下的曲线边缘细节。运行时按当前裁剪区绘制色带，无曲线细分或路径分配；上方渐变继续使用 tiny_gfx 的 8 行有界暂存。

```sh
python3 scripts/generate-folio-wallpaper.py
python3 scripts/generate-folio-wallpaper.py --check
```

其他尺寸由 `WallpaperPainter` 按同一组曲线参数矢量绘制。此资源只有背景几何的填充记录；所有应用和系统图标继续使用独立 SVG 源文件直接矢量渲染。
