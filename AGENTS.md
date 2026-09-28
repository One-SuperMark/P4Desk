# 项目约定

- Git 提交使用中文。
- 保留 tiny-flutter / tiny_gfx 原 Rust UI 路线，设备底层使用 ESP-IDF C HAL。
- 固件使用 ESP-IDF 6.0.2，避免混入全局 6.1 环境。
- USB、Rust 与 Swift 数据结构遵循 docs/protocol-v1.md，字段显式小端编码。
- 屏幕只由一个 display owner 提交；TF 挂载失败不得自动格式化。
- 日志不输出画面、便签正文、快捷键内容、设备序列号或凭据。
- 分别记录构建、刷机和实机验收证据。
