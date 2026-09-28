# Rust UI 移植范围与桌面修正

上游固定为 [`pomelos-on-sale/esp32-rust-ui`](https://github.com/pomelos-on-sale/esp32-rust-ui/tree/0b75870835902cbf950505d1539dbb8f6e0a7197)，提交 `0b75870835902cbf950505d1539dbb8f6e0a7197`，保留 MIT 许可证。

## 2026-09-28 核对结果

用户指出板上界面不像参考项目。对照固定版本源码后，确认首版确实移植了 Rust 绘制和组件框架，但把应用桌面重写成了左侧导航与工具卡片。原项目的图标网格、横向分页、页码圆点、壁纸绘制层和状态栏后台应用入口没有进入首版产品。首版 README 将这些一并描述为“应用桌面移植”，范围表述过宽。

首版保留了 `ActiveApp`、`Arc<Mutex<_>>`、`running_apps`、返回保留状态与结束释放状态的生命周期，但 `PageController` 只声明、没有接到桌面 `PageView`；硬件 `BackListener` 也没有为 7B 补齐触屏结束入口。

## 实际复用与适配

| 上游部分 | P4Desk 实现 | 适配内容 |
| --- | --- | --- |
| `tiny-flutter/src/app.rs` | 同路径，固件实际调用 `App` | 外部校时／USB／同步事件重建、手指按住时推迟树替换、模式切换取消触摸 |
| `tiny-flutter/src/widgets`、`rendering` | 同路径组件与渲染树 | 中文测量／换行、滚动裁剪、分页取消子组件点击、横屏布局 |
| `tiny_gfx` | `crates/tiny-flutter/crates/tiny_gfx` | 保留 RGB565 栅格、路径、透明度与图像缩放；移除旧 AMOLED 的偶数矩形提取限制 |
| 原计算器 | `apps/calculator` | 保留状态和运算，按传入尺寸布局 |
| 原应用生命周期 | `apps/app-launcher/src/launcher_state.rs` | 保留应用实例／后台缓存／结束语义，增加持续计时、便签浏览状态、设备事件与持久化配置 |

固件中的真实调用链是：

```text
C app_main
  → rust_main_entry
  → tiny_flutter::App + app_launcher::build_launcher_ui
  → Widget → RenderBox → Canvas → tiny_gfx RGB565
  → P4Backend::flush → host_lcd_draw_bitmap
  → 唯一 display owner → DSI LCD
```

产品 UI 没有通过 LVGL 绘制。ESP-IDF C 层提供板级驱动、USB、存储与面板提交；Rust 保持 UI 和应用状态的所有权。

## 桌面修正范围

按上游桌面模块的职责恢复 `widgets.rs`、`status_bar.rs` 和 `app_icons.rs`，把原图标槽位、`PageView`、页码圆点、后台应用恢复入口接回真实桌面。尺寸来自 `Size`，适配 1024×600。最初恢复原两页 2×2 布局后，用户提出“一屏放 8 个图标试试”，因此改为 **4 列 × 2 行、每页八项**，按容量动态分组。当前六个应用加上 USB 副屏、关闭屏幕两个已有系统动作入口同屏显示，单页隐藏页码圆点；以后增加到超过八项时继续分页。子应用保留触屏返回与结束入口，以及模拟器的硬件返回／结束事件语义。

时钟、番茄钟、便签、Mac 控制和设备设置是本项目新增功能。参考项目的 Terminal、Counter、Hello 和音乐播放器属于示例应用，没有作为这版桌面工具启用。

USB 副屏与关闭屏幕是既有 `RequestMode`、`Screen` 命令的桌面快捷入口，不创建新的后台应用；进入副屏仍经过 Mac 的权限／捕获／USB 能力验证。关闭屏幕后可轻触唤醒。

状态栏使用实际时间、USB 连接与 TF 状态；7B 没有对应电池／Wi-Fi 信号采样，不沿用上游演示数值。图标和壁纸采用项目自制几何资源，保持原桌面的图标与背景绘制接口；参考项目的 Apple 风格图像、Sierra 照片和商业音乐没有随产品分发。

本次修正只涉及 Rust 桌面层。已获用户确认的画面软件 180° 旋转和 GT911 坐标校正保持当前配置，便签资源协议、持久化与 USB 模式生命周期继续使用现有实现。

构建、触摸回归、预览与上板记录分别见 [验收记录](acceptance.md)。源码或预览检查不替代板上实际交互验收。
