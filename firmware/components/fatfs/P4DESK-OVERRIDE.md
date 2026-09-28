# P4Desk 的 FatFs 局部覆盖

来源：ESP-IDF v6.0.2 的 `components/fatfs`，保留各文件原有许可证。
将 `src/ffconf.h` 的 `FF_FS_EXFAT` 从 0 改为 1；项目 sdkconfig 使用 heap LFN。
同时为未启用的 `CONFIG_FATFS_USE_LABEL` 明确提供 0 默认值，因为 exFAT 路径会在 C 表达式中引用这个宏。
不修改全局 ESP-IDF，也不对 TF 卡执行自动格式化。
ESP-IDF 的分区镜像工具仍从锁定 SDK 路径调用。
