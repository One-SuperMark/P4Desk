# ESP-Hosted / ESP-IDF 6.0.2 适配

`esp_hosted_sdio.c` 源自锁定的 `espressif/esp_hosted 1.4.7` 的
`host/port/src/sdio_wrapper.c`，保留原 Apache-2.0 版权头。

本板 TF 使用 SDMMC slot 0，C6 使用 slot 1，共享控制器。IDF 6.0.2
的旧 API 不再允许重复创建控制器。此局部替换复用已创建的槽位所属控制器，
并将 C6 的失败清理限制在其自身槽位，防止卸载 TF。
引用固定 IDF 版本的私有 `sdmmc_get_slot_handle`，升级 SDK 必须重新核对。
板级 TF 挂载先完成，无线任务随后启动；没有两个初始化任务同时进入该路径。

`firmware/CMakeLists.txt` 还补充 IDF 6 拆分后的 driver 依赖。
不修改 component manager 下载的文件，依赖 hash 保持可复现。

禁用上游 `esp_hosted_host_init.c` 构造函数，由 `p4desk_radio` 工作任务明确调用
`esp_hosted_init()`，避免在 `app_main()` 前抢先初始化 SDMMC。
SDIO 命令错误改为返回错误，`hosted_errors.cmake` 保留源头声明生成 API 副本，
将 transport reconfigure 失败转为返回值，不在 Wi-Fi 初始化入口触发 abort。
此修补不承诺恢复运行过程中所有 Hosted 底层总线故障；驱动本身的致命错误策略
仍需以断线／压力实机测试确认。
