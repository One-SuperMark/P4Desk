#!/usr/bin/env python3
"""Bake pinned DINish tabular lining figures into the flip-clock alpha atlas.

Regeneration uses macOS Swift/CoreText and Python's standard library.
Normal firmware builds embed the checked-in atlas without either dependency.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "assets/clock/flip-digits.alpha"
METADATA = OUTPUT.with_suffix(".alpha.json")
README = OUTPUT.parent / "README.md"
SOURCE_COMMIT = "a5f3b2a3b932336225815bf9005e3b72cc3de71c"
CHARACTERS = "0123456789-"
WIDTH, HEIGHT = 144, 208
STYLES = {
    "heavy": {
        "name": "DINish Heavy", "file": "DINish-Heavy.ttf", "weight": 800,
        "sha256": "6812edbc60553218229b256670b57fdaf8186969e56f9aa7c19bcb19c66ad263",
    },
    "black": {
        "name": "DINish Black", "file": "DINish-Black.ttf", "weight": 900,
        "sha256": "0e42282cd16d1d56de835f8c6e2f8a679aba3678a45ef5e8707acaaa6cb4a8fb",
    },
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def font_tables(data: bytes) -> dict[bytes, bytes]:
    tables = {}
    for index in range(struct.unpack_from(">H", data, 4)[0]):
        tag, _, offset, length = struct.unpack_from(">4sIII", data, 12 + 16 * index)
        tables[tag] = data[offset:offset + length]
    return tables


def validate(font_bytes: bytes, atlas: bytes, metadata: dict, style: dict) -> None:
    tables = font_tables(font_bytes)
    if struct.unpack_from(">H", tables[b"OS/2"], 4)[0] != style["weight"]:
        raise SystemExit("源字体的原生字重与所选样式不一致。")
    if metadata["characters"] != CHARACTERS or (metadata["width"], metadata["height"]) != (WIDTH, HEIGHT):
        raise SystemExit("图集尺寸或字形顺序与 Rust 绘制契约不一致。")
    if len(atlas) != len(CHARACTERS) * WIDTH * HEIGHT:
        raise SystemExit("图集长度与固定 11 字形布局不一致。")
    metric_count = struct.unpack_from(">H", tables[b"hhea"], 34)[0]
    units_per_em = struct.unpack_from(">H", tables[b"head"], 18)[0]
    advances = []
    for index, glyph in enumerate(metadata["glyphs"]):
        cell = atlas[index * WIDTH * HEIGHT:(index + 1) * WIDTH * HEIGHT]
        ink = [(offset % WIDTH, offset // WIDTH) for offset, alpha in enumerate(cell) if alpha]
        bounds = [min(x for x, _ in ink), min(y for _, y in ink),
                  max(x for x, _ in ink) + 1, max(y for _, y in ink) + 1]
        if bounds != glyph["bounds"]:
            raise SystemExit("图集的真实像素与记录的字形边界不一致。")
        if min(bounds[0], bounds[1], WIDTH - bounds[2], HEIGHT - bounds[3]) < 3:
            raise SystemExit("字形被裁切或没有足够的透明边距。")
        if abs((bounds[1] + bounds[3]) / 2 - HEIGHT / 2) > 0.5:
            raise SystemExit("字形实际墨迹中心偏离翻页中轴。")
        if index < 10:
            metric_index = min(glyph["glyph_id"], metric_count - 1)
            advance = struct.unpack_from(">H", tables[b"hmtx"], metric_index * 4)[0]
            advances.append(advance)
            expected = advance / units_per_em * metadata["font_size"]
            if abs(expected - glyph["advance"]) > 0.000001:
                raise SystemExit("原字体字宽与 CoreText 选中的等宽数字不一致。")
    if len(set(advances)) != 1 or advances[0] != 524:
        raise SystemExit("选中的 tnum 数字不是严格等宽字形。")


def make_readme(metadata: dict, style: dict, digest: str, source_url: str) -> str:
    rows = "\n".join(
        f"| `{glyph['character']}` | {index} | {index * WIDTH * HEIGHT} | {glyph['glyph_id']} | "
        f"`{tuple(glyph['bounds'])}` | {glyph['ink_center_y']} | {glyph['optical_y_adjustment']} |"
        for index, glyph in enumerate(metadata["glyphs"])
    )
    return f"""# 翻页时钟数字图集

当前使用 **{style['name']}** 的原生 **{style['weight']}** 字重，启用字体内置 `tnum`（等宽数字）与 `lnum`（齐线数字），保留普通无斜线的 `0`。字形来自源字体，不添加描边或横向拉伸。

## 固定二进制布局

- 字符顺序：`{CHARACTERS}`，11 个字形。
- 每个字形：**{WIDTH} × {HEIGHT}**，单通道 alpha8；0 为透明，255 为完全覆盖。
- 每字形 {WIDTH * HEIGHT:,} 字节，总长度 **{len(CHARACTERS) * WIDTH * HEIGHT:,} 字节**，无文件头或行填充。
- 按字形、从上到下的行、从左到右的像素连续存储：`glyph_index * {WIDTH} * {HEIGHT} + y * {WIDTH} + x`。
- 颜色由 Rust UI 指定；每像素一个字节，无大小端转换。
- 数字的源 `hmtx` advance 全部为 **524 / 1024 字体单位**；在当前 {metadata['font_size']} px 字号下均为 **{metadata['glyphs'][0]['advance']} px**。
- 设备以固定 {WIDTH} px 槽位绘制图集；字符 `-` 也占用同一槽位。

## 光学居中与边界

字号 **{metadata['font_size']} px**，使用原生字重。macOS CoreText 选择 `tnum`／`lnum` 字形，CoreGraphics 将其原始轮廓光栅化为灰阶 alpha8。源 TTF 没有被修改。

每个数字的完整墨迹在固定槽位中水平及垂直居中，中心为 **y=104±0.5 px**，与翻页中轴一致。相对名义基线 {metadata['nominal_baseline']} 的光学偏移见下表。生成器验证字形未切边、四边至少 3 px 透明边距、源字宽和实际选中字形严格等宽。

边界为左、上、右、下；右和下不包含。负的光学偏移表示向上移动。

| 字符 | 索引 | 字节偏移 | 源 glyph ID | 非零墨迹边界 | 墨迹中心 y | 光学 y 偏移 |
|---|---:|---:|---:|---|---:|---:|
{rows}

## 来源与许可

- 官方源：[playbeing/dinish](https://github.com/playbeing/dinish)，固定提交 `{SOURCE_COMMIT}`，字体内部版本 **4.007**。
- 源字体：[下载地址]({source_url})；本地文件 [`{style['file']}`](../fonts/source/{style['file']})。
- 源字体 SHA256：`{style['sha256']}`。
- alpha 图集 SHA256：`{digest}`。
- 字体及派生字形使用 **SIL Open Font License 1.1**，随附 [DINish-OFL.txt](../../third_party/licenses/DINish-OFL.txt) 保留原版权声明。
- 选中字形、边界、字宽与校验值详见 [flip-digits.alpha.json](flip-digits.alpha.json)。

## 重新生成

正常 Rust／固件构建直接内嵌此图集，不需要运行时字体处理、TF、Python 或 Swift。重新生成需要 macOS、Xcode Command Line Tools（Swift／CoreText）和 Python 标准库，不需要 Pillow：

```bash
python3 scripts/generate-clock-assets.py --font {metadata['selected_style']}
python3 scripts/generate-clock-assets.py --font {metadata['selected_style']} --check
```

`--check` 在临时目录生成并逐字节比较，不修改已保存资产。另一个字重使用 `--font {'black' if metadata['selected_style'] == 'heavy' else 'heavy'}`。辅助源为 [rasterize-clock-font.swift](../../scripts/rasterize-clock-font.swift)。升级系统光栅化库后先检查图集边界与校验值。
"""


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--font", choices=STYLES, default="heavy")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    style = STYLES[args.font]
    source = ROOT / "assets/fonts/source" / style["file"]
    font_bytes = source.read_bytes()
    if sha256(font_bytes) != style["sha256"]:
        raise SystemExit("源字体 SHA256 与固定官方版本不一致。")
    source_url = f"https://raw.githubusercontent.com/playbeing/dinish/{SOURCE_COMMIT}/fonts/ttf/DINish/{style['file']}"
    with tempfile.TemporaryDirectory(prefix="p4desk-clock-font-") as directory:
        temporary_atlas = Path(directory) / "digits.alpha"
        subprocess.run([
            "swift", str(ROOT / "scripts/rasterize-clock-font.swift"),
            str(source), str(temporary_atlas),
        ], check=True, stdout=subprocess.DEVNULL)
        atlas = temporary_atlas.read_bytes()
        metadata = json.loads(temporary_atlas.with_suffix(".alpha.json").read_text())
    validate(font_bytes, atlas, metadata, style)
    digest = sha256(atlas)
    metadata.update({
        "selected_style": args.font, "weight": style["weight"], "source_commit": SOURCE_COMMIT,
        "source_url": source_url, "source_sha256": style["sha256"], "atlas_sha256": digest,
        "font_version": "4.007", "license": "SIL OFL 1.1",
    })
    metadata_bytes = (json.dumps(metadata, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()
    readme_bytes = make_readme(metadata, style, digest, source_url).encode()
    generated = [(OUTPUT, atlas), (METADATA, metadata_bytes), (README, readme_bytes)]
    if args.check:
        for path, expected in generated:
            if not path.is_file() or path.read_bytes() != expected:
                raise SystemExit(f"{path.relative_to(ROOT)} 与固定源生成结果不一致。")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        for path, data in generated:
            temporary = path.with_name(path.name + ".tmp")
            temporary.write_bytes(data)
            temporary.replace(path)
    print(json.dumps({
        "font": style["name"], "weight": style["weight"], "features": metadata["features"],
        "source_sha256": style["sha256"], "atlas_sha256": digest, "bytes": len(atlas),
        "font_size": metadata["font_size"], "digit_advance_units": 524,
        "digit_advance_pixels": metadata["glyphs"][0]["advance"],
        "numeric_ink_centers": [glyph["ink_center_y"] for glyph in metadata["glyphs"][:10]],
        "check_passed": args.check,
    }, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
