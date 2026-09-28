# 使用交付包

交付文件 `P4Desk-0.1.0.zip` 解压后，直接使用 `.app` 与已经构建的固件。完整源码也在包内，只有重新编译时需要 Rust／ESP-IDF 环境。

## 包内文件

| 路径 | 用途 |
| --- | --- |
| `P4Desk.app` | Apple Silicon Mac 菜单栏应用；自带字体和字形生成器 |
| `firmware/p4desk.bin` | P4 应用固件 |
| `firmware/bootloader/bootloader.bin` | 与固件配套的 bootloader |
| `firmware/partition_table/partition-table.bin` | 32 MiB Flash 分区表 |
| `firmware/flasher_args.json` | 刷写地址、模式和对应文件 |
| `firmware/p4desk.elf` | 调试／解析固件回溯所用的 ELF |
| `scripts/backup-device.py` | 完整 Flash 与 NVS 备份工具 |
| `scripts/device-tool.py` | 校验备份后刷写、读取限时启动日志 |
| `tools/fontpack/` | 可独立调用的字形工具、字体及许可 |
| `P4Desk-0.1.0-source.tar.gz` | 对应提交的完整 Rust workspace、C 固件和 Swift 工程 |
| `manifest.json` | 对应源码提交、工具链及每个文件的 SHA256 |
| `docs/acceptance.md` | 已验证项目与尚待实机检查的逐项记录 |

旧固件备份属于设备的私有恢复数据，保存在项目的 `backups` 中，不放入交付 ZIP。

## 当前开发板

本轮开发板操作以 [验收记录](acceptance.md) 为准。已刷入对应固件时，直接启动 App 即可；完整备份已经存在时，无需重复读取 32 MiB Flash。

Mac 打开 `P4Desk.app` 后，菜单栏显示连接状态。字体与生成器已包含在 App 中；编辑便签／按钮并点击同步，会自动生成所需中文资源并启用。设备默认 Pad，进入副屏需手动操作。

权限可稍后授予。屏幕录制用于捕获虚拟屏，辅助功能用于触摸与快捷键；应用在手动开启相应功能时请求权限。

## 给另一块 7B 刷写

在解压目录中，选好设备串口和已安装 ESP-IDF 6.0.2 的 Python 后执行：

```sh
python3 scripts/backup-device.py \
  --port /dev/cu.YOUR_P4_SERIAL \
  --python /absolute/path/idf6.0_python_env/bin/python \
  --output /private/backup/p4-before-p4desk.bin

python3 scripts/device-tool.py flash \
  --port /dev/cu.YOUR_P4_SERIAL \
  --python /absolute/path/idf6.0_python_env/bin/python \
  --backup /private/backup/p4-before-p4desk.bin \
  --build-dir ./firmware
```

保留原 TF 卡在板载卡槽；不会格式化 TF。Type-C 供电／串口与 Type-A HS 数据线的区别见 [接线说明](wiring.md)。

## 核对交付文件

`manifest.json` 的 `files` 记录路径、长度和 SHA256。Mac App 可执行以下签名检查：

```sh
codesign --verify --deep --strict P4Desk.app
```

默认 ad hoc 签名适用于本机／个人使用。若改用开发者身份签名，应在第一次授权前确定身份，并在后续版本保持一致。重新签名会改变 App 文件的 SHA256，需要重新生成交付清单。
