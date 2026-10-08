# 整屏横向绕回：时序校正与扫描恢复

## 问题与证据边界

2026-10-08 用户照片显示用量监控的 Home、关闭、导航以及两栏内容一起横向绕回。重新采集 UART 时已经记录一次 POWERON，90 秒内分配失败、Bridge 欠载和 Host DPI FIFO 异常没有增长，但用户反馈画面仍偏移。该窗口没有捕捉到首次异常，不能据此证明原故障不存在，也不能把启动时 `ST0=0x100000` 当作已证明的错位原因。

审查确认：LCD 只有一个提交点；传入精确 framebuffer 基址，stride 固定 1024；DMA 扫描仍为 1024×600，608 行只用于分配容量；迟到或缺失 completion 会失败关闭，不会静默释放扫描缓冲。没有发现能静默产生整屏绕回的基址偏移错误。

## 1. 两端行时序校正

ESP-IDF 6.0.2 的 DPI 分频和 `mipi_dsi_hal_host_dpi_set_horizontal_timing()` 分别取整。原面板宏请求 52 MHz，PLL 240 MHz / 5 实际为 48 MHz；1000 Mbps 的 lane byte clock 为 125 MHz。原 HFP 160 对应输入 HTOTAL 1354，两端取整后的周期不同：

| 配置 | Bridge 行时钟 / 48 MHz | Host byte clocks / 125 MHz | 行周期 |
|---|---:|---:|---|
| 原 HFP 160 | 1250 | 3255 | 26.041667 / 26.040000 µs |
| 新 HFP 158 | 1248 | 3250 | 两端均为 26.000000 µs |

只调整输入 HFP 两个像素时钟，保留 HS 10、HBP 160、宽 1024、高 600 和垂直 total 636。SDK 补偿后的实际 HFP 为 54；理论刷新约 60.474 Hz。启动由 owner 读取实际分频、Bridge active/total、Host HLINE/vertical total 并打印 `line_period_match`，不把配置名称当作实际时钟。

这消除了一项源码可确认的时序风险，尚不能单靠数学模型证明它是照片中首次异常的唯一触发原因。

## 2. 扫描恢复保持所有权

项目本地 DPI 扩展 v6 只读取固定 SHA256 的 ESP-IDF 6.0.2 源码，在构建目录生成补丁；不修改全局 SDK。公开面板 `init/reset` 不适合作为恢复，会重置索引或面板 GPIO，破坏旧的 pending/completion 状态。

恢复由已经绑定的同一个 display owner 完成：

1. 在共享 ISR 锁下请求停止续传。
2. 等真实完整 DMA completion；ISR 不重启 DMA，投递 `A→A`，保留 A 所有权、pending B 和连续 counter。
3. ISR 回调尾部确认停止；任务关闭 Bridge/Host，复位两者并恢复明确列出的 R/W 配置。D-PHY、面板 GPIO 和共享 DMA controller 不复位。
4. 首次重新传输仍从 A 开始；下一真实 `A→B` 才激活 pending B 并释放 A。
5. 恢复期有意断流不计入新的 Bridge 欠载；Host 停流报告由 owner 归档到 `recovery_status0/1`，不混入正常扫描故障增量。

500 ms 等待截止前取消与 ISR commitment 共用锁。若取消先赢，原扫描继续；若停止已经承诺却未完成，owner 保留全部缓冲并停提交，不强制 abort AXI、复用缓冲或重启应用。

触发仅使用 Bridge 欠载或 DPI FIFO overflow/underflow 新增；重复快照不触发。首次立即恢复，随后间隔至少 3 秒，任意滚动 60 秒最多 3 次。普通 D-PHY 启动报告不触发恢复，也没有周期性无条件重置显示器。

## 3. Flash 保存与 XIP 试验的取舍

临时验证版启用 `CONFIG_SPIRAM_XIP_FROM_PSRAM=y`，把指令和只读数据加载到 PSRAM，使 SPI1 Flash 操作走互斥路径，减少持久化保存引起的全局缓存与调度暂停。实机正常监控下 PSRAM 最小空闲约 10.7 MB，扫描恢复成功。

独立审查发现文件预览 PNG / progressive JPEG 的 decoder 中仍有不可恢复分配，现有文件长度和输出尺寸上限不能代表总峰值。XIP 增加约 6.6 MiB 常驻内存后会降低该路径的安全余量，因此最终交付版关闭 XIP。没有把“监控静置时无分配失败”外推为所有文件预览都安全，也没有宣称旧 DMA 在 Flash 保存时必然停传。

最终保留 ISR cache-safe、32 KiB cache 分块、扫描 QoS 15、L2 256 KiB / 128 B、HEX 200 MHz 和内部 RAM 保留池；不一起开启依赖具体 Flash 型号的 auto-suspend。大图预览峰值预算属于独立待补强项。

## 验证与诊断

- `scripts/test-display-scan-health.sh`：ASan/UBSan，11 项及 50,000 轮轮询，覆盖增量、饱和、冷却、滚动预算、时钟边界。
- observer 生成代码测试：执行实际 ISR/API，覆盖 pending 保留、连续 counter、取消与已承诺停止、owner 限制、寄存器恢复和恢复期报告。
- 既有 pipeline、Pad damage、cache sync 回归保持独立验证。
- `CONFIG_P4DESK_LCD_RESYNC_SELF_TEST` 默认关闭；开发版启动 10 秒后仅执行一次恢复，验证后交付版必须关闭。
- `p4desk_scan_timing` 为实际行时序；`p4desk_scan_resync` 为每次恢复及停流报告；`p4desk_display_health` 增加尝试/成功/失败/park 计数。

本轮构建、刷机、实机日志与用户视觉验收分开记录于 `acceptance-display-scan-recovery.json`。不能用主机测试、成功刷写或短时无报错替代实际屏幕位置、触摸及长时间运行验收。

### 本轮已完成的记录

- 最终固件 7,013,056 bytes，SHA256 `126b87726e08717466fe46a70adbd3260b5ef168937730bf888914e1a77fd7d6`；bootloader、分区表、应用三段写入校验通过，未写 NVS / SPIFFS 数据或 TF。
- 15 项 observer 生成代码执行测试、11 项 scan health ASan/UBSan 测试通过，既有 pipeline、Pad damage、cache sync 回归通过。
- 临时 XIP 验证版完成一次真实扫描恢复，耗时 16,736 µs，stop / resume 均成功，counter 连续；90.2 秒记录中没有恢复链、欠载增长或 FIFO 错误。
- 最终关闭 XIP / 启动自测，180.45 秒记录中 5 个诊断样本，末次 uptime 150 秒：Bridge 欠载、DPI overflow/underflow、分配失败、cache / PPA 错误及 panic 为 0。自然故障恢复触发次数为 0，不能据此声称已在自然故障现场验证自动恢复。
- 实际启动寄存器为 48 MHz、Bridge H 1248 / V 636、Host HLINE 3250 / V 636，`line_period_match=1`；有效区 1024×600。DMA 最大边界间隔 16,657 µs，超过 25 ms 的次数为 0。
- 最小内部空闲 67,619 bytes，最小 PSRAM 空闲 17,570,320 bytes；34 次取数成功及 3 次状态保存成功。一次 Offline 发生在启动联网前，后续成功。
- 普通 Host 启动报告 `ST0=0x100000` 出现一次，2290 次累计采样中后续未增加；不能称全部 DSI 状态为 0。
- 用户于 2026-10-08 反馈“已经恢复了”，确认当前画面恢复；本次没有单独确认触摸或长期稳定性。USB 副屏以及大图预览没有重新完成全量压力验收。
