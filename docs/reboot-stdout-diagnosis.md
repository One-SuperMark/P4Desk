# Rust 标准输出失败导致启动重启循环

## 2026-09-30 实机证据

用户反馈设备只接 Mac，持续闪屏重启。被动读取串口 50 秒，未发送复位指令、未刷写或清除存储。捕获 19 次相同 panic，相邻 panic 的间隔中位数为 2.545 秒。

```text
p4desk: Rust UI starting
thread '<unnamed>' (1) panicked at .../std/src/io/stdio.rs:1209:9:
failed printing to stdout: No such file or directory (os error 2)
abort() was called at PC 0x400eeda3 on core 0
Rebooting...
rst:0xc (SW_CPU_RESET),boot:0xf (SPI_FAST_FLASH_BOOT)
p4desk: boot reset_reason=4
```

板上固件报告 ESP-IDF **6.0.2**，应用编译时间为 `Sep 30 2026 13:51:31`，ELF SHA256 前缀为 `2e155e270`。采集初始启动原因为 POWERON；之后观察到 19 次软件复位，其中 18 次在采集结束前进入应用并报告 `ESP_RST_PANIC`（4）。每次崩溃前，PSRAM 自检、TF 挂载与读写、`p4settings` 挂载与读写均通过。

本次重启循环的直接原因是 Rust `println!` 将标准输出 IO 错误升级为 panic，固件的 `panic=abort` 随后导致重启。源码中，启动恢复会话时会执行该日志调用；自动保存线程及 UI 性能统计也有相同风险。日志中的 ENOENT 描述的是**标准输出失败**，不能据此认定便签、会话文件或 TF 数据损坏。

当前连接经板载 USB 转串口，原生 USB Serial/JTAG 并未连接。恢复固定版本 SDK 后，确认了控制台路由问题：

1. [ESP-IDF 6.0.2 的 console_open](https://github.com/espressif/esp-idf/blob/v6.0.2/components/esp_stdio/stdio_vfs.c) 在创建 C 标准流前先打开主 UART 和辅助 USB 通道，占用底层文件描述符。
2. [固定 Rust 工具链的 stdout](https://github.com/rust-lang/rust/blob/75a75c3e0/library/std/src/sys/stdio/unix.rs) 按 `STDOUT_FILENO=1` 写入，不读取 C `stdout` 实际的描述符。双控制台配置下，描述符 1 可指向辅助 USB。
3. [6.0.2 USB Serial/JTAG 写入函数](https://github.com/espressif/esp-idf/blob/v6.0.2/components/esp_driver_usb_serial_jtag/src/usb_serial_jtag_vfs.c) 在原生 USB 未连接时直接返回 `-1`，且没有设置 `errno`（源码标注 `IDF-14303`）。因此 Rust 可能报告之前文件访问留下的 ENOENT；这不是文件损坏的证据。

新固件实机启动确认 `stdout_fd=3 stderr_fd=3 native_usb_host=0`，Rust 诊断通过 C 日志入口正常输出，符合上述路由问题。旧固件对应的 ELF 当前不在本机，未对旧崩溃地址符号化，不把某个具体恢复分支认定为已由堆栈证实。此前 [电源切换重启](power-handover-diagnosis.md) 的独立问题继续保留原结论。

## 源码修正

固件 Rust 运行时的 10 处 `println!` 改用统一 `diagnostic!`。设备端在 512 字节栈缓冲内格式化，按 UTF-8 字符边界截断超长信息，通过 C HAL 的 `p4desk_log_diagnostic` 调用 `ESP_LOGI`，与现有启动和健康日志共用实际 C 控制台路由。该路径不调用 Rust stdout，不为输出分配堆内存，也不因日志 IO 失败触发 Rust panic。C 端再次限制长度，不把消息内容作为格式字符串。

主机测试后端使用 `Write` 返回值处理控制台 IO 失败；失败日志允许丢弃，不通过同一失败通道递归报告或增加应用层重试。现有日志的状态、数量、耗时字段保留，设备输出增加 `p4desk_rust` 标签。主机工具及构建脚本的标准输出不变。

主机故障测试注入实机出现的 ENOENT（2），并覆盖 EIO（5）、EBADF（9）和输出容量不足；另验证正常日志格式、固件缓冲区格式化、512 字节边界与 UTF-8 截断。目标板的路由、输出和持续运行另行验收。

使用项目固定 `nightly-2026-09-27`，`cargo test --locked -p rust_main --lib` 的 **35 项测试通过**；另外将诊断模块以 `-C panic=abort -Z panic-abort-tests` 编译，**6 项日志测试通过**。测试目标为本机 `aarch64-apple-darwin`；Rust 工具链与依赖放在项目忽略目录 `.cache/reboot-diagnosis`。

## 构建、刷机与实机验收

分别记录在 [本次验收数据](acceptance-reboot-stdout.json)。原项目记录的 `/Volumes/work` 构建盘未挂载，因此在 `/Users/yanglinghui/esp/esp-idf-v6.0.2` 建立独立 SDK，固定官方 `v6.0.2` / `7101770dc6db2667b3c477cc31365dd1acd6db4e` 及其子模块提交，创建 `idf6.0_py3.12_env`。已有 6.1 SDK 保持独立；复用的 GCC 版本与 6.0.2 官方要求相同。构建目录为 `.cache/reboot-diagnosis/build`。

本次结果：

- **构建通过**：应用 5,431,232 字节，SHA256 `37250240cca04b9fa67be6afb0e79a254c1f2960bc885aef1963078235800e9b`；ELF SHA256 `b122030b691b2de6eda3d9a459fed80f6d691990e0acdd20edc0d9da3221f9e3`。镜像检查确认为 ESP-IDF 6.0.2，芯片修订范围 3.0–3.99。
- **备份完成**：`backups/reboot-20260930/p4-before-stdout-fix.bin`，完整 32 MiB，32 个分块均经设备 MD5 与本地 SHA256 校验；其中 13 个全 `0xFF` 分块先核对设备 MD5 再重建。另存独立 NVS。高速串口读取不稳定，本次使用 230400 波特率及单个 4 KiB 在途读取帧。
- **刷写通过**：`device-tool.py flash` 再次核对完整备份 SHA256 后写入三个镜像，三个写入 Hash 均通过。分区表与设备备份一致，刷写区间不覆盖 NVS 和已有存储分区，未格式化 TF。
- **实机观察**：新 ELF 启动，成功恢复 2 个应用、4 个最近入口与历史时间，计时器按约定暂停；TF 和设置区读写正常，5 次自动保存成功，Wi-Fi 网络对时成功。最后健康记录为运行 150 秒、提交 445 帧、解析错误 0；保存线程最小剩余栈 1,888 字节。观测期间没有 panic、看门狗或循环重启。
- **用户确认与观察边界**：原计划采集 180 秒，USB 串口在约 158 秒时断开；用户确认「刚拔线或换接口，界面正常」。本次启动重启修复已构建、刷入并由日志与用户界面反馈验证；没有将采集记为完整 180 秒。触摸未单独确认，该记录不替代长时间使用、完全断电及电源切换验收。
