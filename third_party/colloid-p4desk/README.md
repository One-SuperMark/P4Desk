# Colloid 图标来源

从用户提供的 `/Volumes/work/workspace/Colloid-SVG/P4Desk精选/` 导入。
上游为 `vinceliuice/Colloid-icon-theme`，固定提交
`ceac6608ecd0e40025cbc2ebbd32bf0e0f4ebc6a`。

- `dark/`、`light/`：各 21 个实际使用的 SVG，原样复制；保留各自 manifest 和 GPL-3.0 LICENSE。
- `manifest.json`、`SOURCE-README.md`、`theme-palettes.json`：原精选包的元数据、说明与颜色映射。原清单包含 4 个备选图标；这些备选项没有导入固件。
- `assets/colloid/`：供 Rust 编译的规范化派生 SVG；保持完整 64×64 画布比例，坐标统一放大至 128×128。
- `scripts/generate-colloid-icons.py`：检查输入 SHA256，展开变换、椭圆、旋转圆角矩形和渐变引用；保留透明度，跳过源中隐藏的残留装饰。
- 时钟仅删除经过数量校验的 4 层固定指针／阴影，由设备实时绘制指针；其余轮廓保留。
- USB 的矩形裁剪在离线转换中验证包围所有绘制内容后消除。Office 渐变坐标的上游舍入误差在 128 px 画布上小于 0.003 px，规范化为垂直渐变。

深浅版本是用户精选包的派生配色；不将其声明为上游官方独立主题。派生几何保留 GPL-3.0 标识，不改标为项目原创图标的 MIT 许可。构建不需要用户的原始绝对路径。
