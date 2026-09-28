# P4Desk 字库工具

此工具只在 Mac 上解析完整字体，把当前便签、标题和按钮文字烘焙成 P4F1 字形包。固件使用内嵌 UI 子集和 TF 上的按需字形缓存，避免在 P4 上解析完整中文字库。

默认字体是仓库中已附带许可的 `assets/fonts/NotoSansSC-Regular.otf`。打包进 Mac 应用后，通过 `--font` 显式传入 Resources 内字体路径。

```sh
cargo build -p p4desk-fontpack --release
./target/release/p4desk-fontpack bake --font assets/fonts/NotoSansSC-Regular.otf --snapshot snapshot.json --output notes.p4f --sizes 18,22,28,36
./target/release/p4desk-fontpack validate --input notes.p4f --snapshot snapshot.json
./target/release/p4desk-fontpack --font assets/fonts/NotoSansSC-Regular.otf --text-file assets/generated/ui-text.txt --output assets/generated/ui.p4f --sizes 18,22,28,36
```

`--snapshot` 接受 Snapshot JSON 或带 `state` 字段的协议消息，提取 `notes[].title/body` 和 `buttons[].label`。也可使用 UTF-8 `--text-file`。默认字形包含 ASCII 和计算器符号。缺少源字体字形或输出超过 8 MiB 时失败，不输出原文。成功 stdout 为 `valid/bytes/glyph_count/sizes` JSON；失败 stderr 为 `valid/error` JSON。

## P4F1 文件布局

全部数值显式小端，UTF-32 Unicode 标量码位，位图为 alpha8。记录按 `(字号, 码位)` 严格升序。字号是固定像素整数；设备所需字号为 18、22、28、36。

| Header 偏移 | 字段 |
| --- | --- |
| 0 | `P4F1` 4 字节 magic |
| 4 | u16 版本 1 |
| 6 | u16 记录长度 32 |
| 8 | u32 字形数 |
| 12 | u32 记录表起点，固定 32 |
| 16 | u32 位图池起点 |
| 20 | u32 整个文件长度，最多 8 MiB |
| 24、28 | u32 保留，必须为 0 |

| 每条记录偏移 | 字段 |
| --- | --- |
| 0 | u32 Unicode 码位 |
| 4、6、8 | u16 字号、宽、高 |
| 10、12 | i16 xmin、ymin 基线偏移 |
| 14 | u16 保留，必须为 0 |
| 16 | f32 advance，有限且非负 |
| 20 | u32 alpha8 位图的文件绝对偏移 |
| 24 | u32 位图长度，必须为宽×高 |
| 28 | u32 保留，必须为 0 |

设备先验证 SHA-256、表边界、Unicode 码位、字号、排序、位图范围以及全部文字的字号覆盖，再将便签与字库作为一个代次启用。文件 provider 只读表和按需位图，位图缓存最多 512 KiB / 512 项。持有的字形用 Arc 保证代次切换后的生命周期安全。
