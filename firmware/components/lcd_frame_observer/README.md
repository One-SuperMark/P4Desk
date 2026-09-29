# DPI DMA 帧观察接口

本组件在项目构建目录生成 ESP-IDF 6.0.2 的 DPI 驱动扩展，替换 `__idf_esp_lcd` 中的一份 `esp_lcd_panel_dpi.c`。全局 SDK 文件只读。原源码 SHA256 固定为 `7a5acacd2be4f4560b2ae85e2414aa3c5c36d313f58e62208e24539ac33e607a`，同时验证十处修改锚点唯一；任意源码变化都会中止配置。生成文件保留原 Apache-2.0 头和文件名，原 `esp_lcd/linker.lf` 的 ISR 映射继续生效。

## 接入

在使用该接口的组件中加入 `PRIV_REQUIRES lcd_frame_observer`，并包含 `p4desk_lcd_frame_observer.h`。首次注册应使用长期有效的静态 context；ISR 只记录完成事件或唤醒 display owner。注册与注销必须发生在 task 中，不能与面板 init/delete 并发。

每次 DW-GDMA full-transfer 完成时，驱动保留上次真正启动的 `scanning_fb_index`，读取当前待提交索引，用原路径重新启动 DMA，然后在锁外调用观察回调。事件内容：

- `completed_fb` / `completed_index`：此次完整 DMA 读取的源缓冲。
- `next_fb` / `next_index`：此次实际重新启动 DMA 使用的源缓冲。
- `counter`：每次 full-transfer 都加一，包括重复扫描同一缓冲；面板初始化归零，无符号溢出时回绕。

`completed_fb == next_fb` 时，缓冲仍被 DMA 读取，不能回收。仅在完成缓冲与下一缓冲不同、且显示 owner 的 pending/scanning 状态匹配事件时，才能回收旧缓冲。缓冲选择仍由唯一 display owner 调用原 `esp_lcd_panel_draw_bitmap` 完成；原 cache writeback 后才发布索引的顺序保留。

事件代表源内存的 DMA 所有权边界，不代表玻璃上的光学显示完成。原 `on_refresh_done` / VSYNC 调用路径、顺序与中断行为保留，可继续用于显示统计。观察回调在原 DMA restart 之后、原非 VSYNC fallback refresh 回调之前执行。

callback/context 的注册与 ISR 快照使用同一 `portMUX`。回调在锁外运行，因此替换/注销后可能还有一次旧快照的回调；旧 context 必须存活到 DMA 停止或面板删除。本接口没有同步等待 ISR 退出的注销功能。`CONFIG_LCD_DSI_ISR_CACHE_SAFE` 开启时沿用原驱动的 IRAM/internal-RAM 检查。组件要求 DPI 对象分配在内部 RAM，保证 ISR 锁可访问。

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
