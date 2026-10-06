# DPI DMA 帧观察接口

本组件在项目构建目录生成 ESP-IDF 6.0.2 的 DPI 驱动扩展，替换 `__idf_esp_lcd` 中的一份 `esp_lcd_panel_dpi.c`。全局 SDK 文件只读。原源码 SHA256 固定为 `7a5acacd2be4f4560b2ae85e2414aa3c5c36d313f58e62208e24539ac33e607a`，同时验证所有修改锚点唯一；任意源码变化都会中止配置。生成文件保留原 Apache-2.0 头和文件名，原 `esp_lcd/linker.lf` 的 ISR 映射继续生效。

## 接入

在使用该接口的组件中加入 `PRIV_REQUIRES lcd_frame_observer`，并包含 `p4desk_lcd_frame_observer.h`。首次注册应使用长期有效的静态 context；ISR 只记录完成事件或唤醒 display owner。注册与注销必须发生在 task 中，不能与面板 init/delete 并发。

每次 DW-GDMA full-transfer 完成时，驱动保留上次真正启动的 `scanning_fb_index`，读取当前待提交索引，用原路径重新启动 DMA，然后在锁外调用观察回调。事件内容：

- `completed_fb` / `completed_index`：此次完整 DMA 读取的源缓冲。
- `next_fb` / `next_index`：此次实际重新启动 DMA 使用的源缓冲。
- `counter`：每次 full-transfer 都加一，包括重复扫描同一缓冲；面板初始化归零，无符号溢出时回绕。

`completed_fb == next_fb` 时，缓冲仍被 DMA 读取，不能回收。仅在完成缓冲与下一缓冲不同、且显示 owner 的 pending/scanning 状态匹配事件时，才能回收旧缓冲。缓冲选择仍由唯一 display owner 调用原 `esp_lcd_panel_draw_bitmap` 完成；原 cache writeback 后才发布索引的顺序保留。

事件代表源内存的 DMA 所有权边界，不代表玻璃上的光学显示完成。原 `on_refresh_done` / VSYNC 调用路径、顺序与中断行为保留，可继续用于显示统计。观察回调在原 DMA restart 之后、原非 VSYNC fallback refresh 回调之前执行。

callback/context 的注册与 ISR 快照使用同一 `portMUX`。回调在锁外运行，因此替换/注销后可能还有一次旧快照的回调；旧 context 必须存活到 DMA 停止或面板删除。本接口没有同步等待 ISR 退出的注销功能。`CONFIG_LCD_DSI_ISR_CACHE_SAFE` 开启时沿用原驱动的 IRAM/internal-RAM 检查。组件要求 DPI 对象分配在内部 RAM，保证 ISR 锁可访问。

## 显示欠载诊断

补丁 v5 保留 v3 的 DPI bridge underrun 饱和计数、首次／最近单调时间戳。ISR 只读取 IRAM 中的 `esp_timer_get_time` 并在短临界区更新固定大小数据，不打印、不分配、不复位；原中断清除、VSYNC 回调和 yield 路径保留。构建给实际编译生成驱动的 `esp_lcd` 目标增加私有 `esp_timer` 依赖。

v4 对原有呈现 writeback 原位计时并检查返回值，失败时不发布新帧索引，不增加重复 cache 操作。`p4desk_lcd_cache_stats` 返回调用次数、错误次数、最大／累计调用耗时及最近错误码。耗时包含分块过程中的调度时间，不能解释成中断关闭时长。display owner 另记录 DMA 边界最大间隔、超过 25 ms 的间隔次数与边界队列峰值；PPA 记录 SDK prepare／cache／submit 的组合耗时及提交后的完成等待，不能解释成纯 cache 时间。

固件启用 `CONFIG_LCD_DSI_ISR_CACHE_SAFE`，使自动保存 SPIFFS 时 LCD／DW-GDMA ISR 继续执行。仅把 ISR 函数放在 IRAM 不等价于此选项；还需要内部 RAM 中的驱动对象、队列和回调依赖。`CONFIG_ESP_MM_CACHE_MSYNC_C2M_CHUNKED_OPS_MAX_LEN=0x8000` 将完整帧的 C2M 分成 32 KiB 块，让 DMA ISR 可在块间响应。Pad 源保持 owner 锁、目标保持未发布，ISR 不访问这两个正在同步的缓冲，因此不会在块间修改它们。这两项用于修复确定的配置风险，绿屏的具体触发原因仍须结合实机日志和用户观察验证。

`p4desk_lcd_underrun_stats(panel, &stats)` 是 task 中的只读一致快照，失败时归零，不清空累计计数。计数从 panel 创建开始累计，达到 `UINT32_MAX` 后保持饱和，最近时间戳仍更新；`count == 0` 才表示没有观测到欠载。调用不能与面板初始化或销毁并发。

### 完整观察 bridge 与 Host 报告

[ESP32-P4 TRM 第 43 章](https://documentation.espressif.com/esp32-p4_technical_reference_manual_en.pdf)的 PDF 第 2675 页（Register 43.15）说明：当当前行数小于 `FIFO_UNDERRUN_DISCARD_VCNT` 时，bridge underrun 中断被抑制。固定 SDK 原值取水平宽度 1024，超过本屏总行数 636；v5 将诊断阈值改为 0，避免把整个画面范围内的 underrun 都屏蔽。未改变图像尺寸、行时序、颜色或 DMA 传输量。开启完整诊断可能记录上电过渡中的欠载，需按首次正常呈现时间区分；原版本中的零计数不能用于排除显示链路异常。

TRM PDF 第 2642 页（43.4.2.4）明确 Host `INT_ST0/1` 读后清零。唯一 display owner 调用 `p4desk_lcd_host_errors_poll(panel)` 读取并累积报告，首次调用绑定该 task，其它 task 和 ISR 调用在读寄存器之前被拒绝。日志 task 使用 `p4desk_lcd_host_error_stats(panel, &stats)` 读取 RAM 快照，不再访问硬件报告。统计为采样次数、含错误的采样次数、累计状态位、包含 FIFO overflow（bit 7）／underflow（bit 19）的采样次数及首次／最近错误时间。多次硬件事件可能合并成一次报告，次数不能解释为精确事件总数。

TRM PDF 第 2661 页（Table 43.5-2）对这些 DPI FIFO 错误建议复位 Host 并重新发送，但本诊断 API 不复位、不修改中断掩码，不改变帧回收和所有权。发生报告后是否恢复由唯一 owner 的生命周期逻辑决定。Host 错误采样保持在 task，内部 RAM 中的统计不新增 ISR 的非 IRAM 调用。

board C HAL 的 `board_p4_log_display_diagnostics()` 由 UI task 每 30 秒调用，内部再次限频，输出 `display underruns` 累计次数、本期新增、首次／最近发生时间及距最近事件的毫秒数。即使没有事件也输出零计数；零计数时 `last_age_ms=-1`。这个日志取代中断逐次打印，避免闪屏时日志风暴进一步占用 CPU；它只提供关联证据，不证明欠载就是当前闪屏原因，也不自动恢复显示。它不输出帧内容、凭据或缓冲地址。

## JPEG 解码容量

每份帧缓冲按 `ceil(物理高度 / 16) * 16` 行分配，并把容量向上取整到 PSRAM cache alignment。使用相同对齐参数的 `heap_caps_aligned_calloc` 分配，检查加法和乘法溢出，并记录保证可写的 `fb_capacity`。物理高度、`fb_size`、DMA link-list 长度、draw 范围和原 cache 提交的行几何保持不变。

1024×600 RGB565 的扫描大小仍为 1,228,800 字节，每缓冲容量为 1,245,184 字节（608 行），多 16,384 字节；三缓冲合计增加 49,152 字节（48 KiB）。额外行不被 LCD 扫描，也不进入 Pad 的可见区域。

`p4desk_lcd_frame_buffer_capacity(panel, fb, &capacity)` 要求 `fb` **恰好等于**该 panel 的一个 `fbs` 成员。偏移指针和其它分配均被拒绝，失败时输出归零。容量查询不授予写入权；调用者必须从唯一 display owner 取得 FREE/BUILDING 缓冲。

彩色 JPEG 可把此容量传给 `jpeg_decoder_process`，并始终提供非空 `out_size` 让驱动检查实际输出大小。成功后验证输出字节数与 JPEG MCU 尺寸计算一致，再按原 600 行提交。JPEG 驱动负责输出缓存的前后 invalidation。只有主机已按可见 1024×600 区域预旋转的 session 才适合直接解码；Pad 旋转和触摸不受容量扩展影响。灰度 JPEG 在 ESP-IDF 6.0.2 中不能直接转换为 RGB565，仍需原有灰度转换路径。直接解码与会话能力控制由 runtime/主机实现，本组件仅提供内存容量与 DMA 所有权观察。

## 局部验证

可在独立临时目录生成补丁，不改 SDK、不启动设备：

```sh
python3 firmware/components/lcd_frame_observer/tools/generate_patch.py \
  --input "$IDF_PATH/components/esp_lcd/dsi/esp_lcd_panel_dpi.c" \
  --output /tmp/p4desk-dpi/esp_lcd_panel_dpi.c \
  --manifest /tmp/p4desk-dpi/patch-manifest.json
```

标准测试 discovery 与直接指定 SDK 均可使用：

```sh
python3 -m unittest discover -s firmware/components/lcd_frame_observer/tests -v

python3 firmware/components/lcd_frame_observer/tests/test_observer_patch.py \
  --idf-path /Volumes/work/esp/esp-idf-v6.0.2 -v
```

测试优先读取 `IDF_PATH`；未设置时使用本机固定路径 `/Volumes/work/esp/esp-idf-v6.0.2`。直接执行的 `--idf-path` 会覆盖以上两者。缺少 SDK 源码时明确报告 `SkipTest`，提示设置路径；存在但 SHA256 不匹配时仍然失败，避免把其它 SDK 当作已验证的 6.0.2。

完整固件构建与真实显示流水线验收由项目统一执行。该组件生成成功或单文件编译成功不构成 60 FPS 的实机证明。
