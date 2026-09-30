# P4Desk 精选 Numix Circle 资源

来源是用户提供的 `/Volumes/work/workspace/Numix-Circle-SVG/P4Desk精选/`，上游仓库为 `numixproject/numix-icon-theme-circle`，固定提交 `6d4a4aad60994e688ab9dc8f021cb4ec86e9e532`。

本目录保存深浅两套各 21 个原始 SVG、原始主题清单、选择清单、配色表和 GPL-3.0-or-later 许可证。主题清单保留原包的 25 项；其中 `alternatives` 分组每套 4 项没有复制和启用，其余源文件 SHA256 已与清单核对。原始 SVG 本身未改写。

派生修改在 `assets/numix/` 中完成：统一 128×128 画布、展开样式与变换、弧线转曲线、规范化当前资源中的渐变；完整圆形路径转为圆形图元；时钟去除固定指针和指针阴影，以便运行时绘制当前时间。规范化 SVG 与 `numix_icons_generated.rs` 派生几何继续使用 GPL-3.0-or-later，不改标为项目 MIT 许可。

使用方法、复现命令与验证边界见 [项目说明](../../docs/numix-icons-entries.md)。
