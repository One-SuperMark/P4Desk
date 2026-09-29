# 固件第三方来源

- 最小 7B 硬件初始化参考 Waveshare `esp32_p4_wifi6_touch_lcd_7b` BSP 3.0.1，Apache-2.0；保留来源文件 SPDX 声明。未引入 BSP 的 LVGL 适配层。
- EK79007、GT911 与 `esp_lcd_touch` 由 Espressif Component Registry 固定版本获取，保留 managed components 的原许可证。
- TinyUSB 配置和 USB descriptor 写法参考官方 TinyUSB 示例与微雪 `12_usb_extend_screen`，TinyUSB MIT 许可证保留在组件中。
- Rust/C 共享 ABI 构建参考 [esp-rs/esp-idf-template 的 CMake 模板](https://github.com/esp-rs/esp-idf-template/blob/master/README-cmake.md)。上游允许生成的模板以 MIT-0 使用。
- `components/fatfs` 是 ESP-IDF 6.0.2 的项目局部覆盖，保留 FatFs 与 Espressif 各文件原始许可证，仅启用 exFAT；详见该组件内的说明。
- `components/lcd_frame_observer` 在构建目录生成固定 ESP-IDF 6.0.2 `esp_lcd_panel_dpi.c` 的项目扩展，保留原 Apache-2.0 头和来源 SHA256；增加 DMA 源缓冲观察与仅分配容量的 MCU 行填充，不修改全局 SDK。补丁生成器与本项目接口使用 MIT。
- `components/jpeg_full_range` 在构建目录生成固定 ESP-IDF 6.0.2 `jpeg_decode.c` 的项目副本，保留原 Apache-2.0 头、basename 和来源 SHA256；仅为 JPEG BT.601 RGB 输出配置 JFIF 全范围转换矩阵，不修改全局 SDK，硬件颜色验证待完成。生成器、构建配置和项目测试使用 MIT。
