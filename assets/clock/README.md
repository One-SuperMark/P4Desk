# 翻页时钟数字图集

当前使用 **DINish Heavy** 的原生 **800** 字重，启用字体内置 `tnum`（等宽数字）与 `lnum`（齐线数字），保留普通无斜线的 `0`。字形来自源字体，不添加描边或横向拉伸。

## 固定二进制布局

- 字符顺序：`0123456789-`，11 个字形。
- 每个字形：**144 × 208**，单通道 alpha8；0 为透明，255 为完全覆盖。
- 每字形 29,952 字节，总长度 **329,472 字节**，无文件头或行填充。
- 按字形、从上到下的行、从左到右的像素连续存储：`glyph_index * 144 * 208 + y * 144 + x`。
- 颜色由 Rust UI 指定；每像素一个字节，无大小端转换。
- 数字的源 `hmtx` advance 全部为 **524 / 1024 字体单位**；在当前 282 px 字号下均为 **144.3046875 px**。
- 设备以固定 144 px 槽位绘制图集；字符 `-` 也占用同一槽位。

## 光学居中与边界

字号 **282 px**，使用原生字重。macOS CoreText 选择 `tnum`／`lnum` 字形，CoreGraphics 将其原始轮廓光栅化为灰阶 alpha8。源 TTF 没有被修改。

每个数字的完整墨迹在固定槽位中水平及垂直居中，中心为 **y=104±0.5 px**，与翻页中轴一致。相对名义基线 201 的光学偏移见下表。生成器验证字形未切边、四边至少 3 px 透明边距、源字宽和实际选中字形严格等宽。

边界为左、上、右、下；右和下不包含。负的光学偏移表示向上移动。

| 字符 | 索引 | 字节偏移 | 源 glyph ID | 非零墨迹边界 | 墨迹中心 y | 光学 y 偏移 |
|---|---:|---:|---:|---|---:|---:|
| `0` | 0 | 0 | 23 | `(11, 3, 133, 205)` | 104 | 0 |
| `1` | 1 | 29952 | 29 | `(26, 6, 117, 202)` | 104 | 0 |
| `2` | 2 | 59904 | 35 | `(12, 5, 132, 203)` | 104 | 2 |
| `3` | 3 | 89856 | 41 | `(12, 3, 132, 205)` | 104 | 0 |
| `4` | 4 | 119808 | 47 | `(6, 6, 138, 201)` | 103.5 | 0 |
| `5` | 5 | 149760 | 53 | `(13, 4, 131, 203)` | 103.5 | -2 |
| `6` | 6 | 179712 | 59 | `(9, 4, 135, 203)` | 103.5 | -2 |
| `7` | 7 | 209664 | 65 | `(13, 6, 130, 201)` | 103.5 | 0 |
| `8` | 8 | 239616 | 71 | `(9, 3, 134, 205)` | 104 | 0 |
| `9` | 9 | 269568 | 77 | `(9, 5, 135, 203)` | 104 | 2 |
| `-` | 10 | 299520 | 16 | `(32, 85, 111, 123)` | 104 | 0 |

## 来源与许可

- 官方源：[playbeing/dinish](https://github.com/playbeing/dinish)，固定提交 `a5f3b2a3b932336225815bf9005e3b72cc3de71c`，字体内部版本 **4.007**。
- 源字体：[下载地址](https://raw.githubusercontent.com/playbeing/dinish/a5f3b2a3b932336225815bf9005e3b72cc3de71c/fonts/ttf/DINish/DINish-Heavy.ttf)；本地文件 [`DINish-Heavy.ttf`](../fonts/source/DINish-Heavy.ttf)。
- 源字体 SHA256：`6812edbc60553218229b256670b57fdaf8186969e56f9aa7c19bcb19c66ad263`。
- alpha 图集 SHA256：`be04eb78bccb5252f2c8ca99225a61ee9a330689d25a316b69597710871aac1a`。
- 字体及派生字形使用 **SIL Open Font License 1.1**，随附 [DINish-OFL.txt](../../third_party/licenses/DINish-OFL.txt) 保留原版权声明。
- 选中字形、边界、字宽与校验值详见 [flip-digits.alpha.json](flip-digits.alpha.json)。

## 重新生成

正常 Rust／固件构建直接内嵌此图集，不需要运行时字体处理、TF、Python 或 Swift。重新生成需要 macOS、Xcode Command Line Tools（Swift／CoreText）和 Python 标准库，不需要 Pillow：

```bash
python3 scripts/generate-clock-assets.py --font heavy
python3 scripts/generate-clock-assets.py --font heavy --check
```

`--check` 在临时目录生成并逐字节比较，不修改已保存资产。另一个字重使用 `--font black`。辅助源为 [rasterize-clock-font.swift](../../scripts/rasterize-clock-font.swift)。升级系统光栅化库后先检查图集边界与校验值。
