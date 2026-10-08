# Pad 界面字体

## 选择与来源

2026-09-29 用户要求将界面 typeface 调整为苹方或相近字体。当前选择 **HarmonyOS Sans SC Regular**，风格比较依据为本项目 1024×600 桌面与工具页面的同字号合成预览；“接近苹方”是视觉判断，未将它称为苹方字形。

苹果的 [macOS 字体条款](https://www.apple.com/legal/sla/docs/macOSTahoe.pdf)要求依据字体附带的嵌入限制使用，不能据系统已安装直接推定可随独立固件分发。本项目采用嵌入和应用分发条件明确的 [HarmonyOS Sans Fonts License Agreement](https://github.com/openharmony/utils_system_resources/blob/4d96c1c7158103732b687e9ea47f4454f6df3ea4/LICENSE_Fonts)，按该许可保留版权、完整协议和软件内字体名称声明。

- 官方来源：OpenHarmony `utils_system_resources`，固定提交 `4d96c1c7158103732b687e9ea47f4454f6df3ea4`。
- 中文／ASCII 系统子集与默认同步字体：`HarmonyOS_Sans_SC_Regular.ttf`，静态 1.9 / 原生 400 字重。
- 普通拉丁回退：同源 `HarmonyOS_Sans_Regular.ttf`，静态 1.9 / 原生 400 字重。
- 完整 TTF 文件与轮廓保持原样；Rust 字形工具只生成原字形的 alpha8 图像。来源、大小和 SHA256 记录在 `assets/fonts/SOURCES.json`。
- 完整中文字体随 P4Desk Mac 应用保留原许可分发；独立工具目录通过 `--font` 引用应用 Resources 中的字体，不再重复打包字体文件。

翻页时钟大数字使用此前已验收的 DINish Heavy 图集。它属于独立的数字控件；日期、状态、按钮与其他界面文字使用当前系统字体。

## 系统文字与用户内容

`Font::default_font()` 用于桌面、状态栏、设置、计算器与系统提示。内置 UI 字形优先，TF 上旧便签字库不会覆盖系统标签。

`Font::content_font()` 用于便签标题／正文与快捷按钮标签。它优先使用当前同步代次字形，然后逐字回退到 UI 子集、HarmonyOS 拉丁字形和缺字框。字体测量与绘制使用同一解析路径；两种字体共享有界的拉丁缓存。旧 TF 数据继续按原有效代次读取，新同步完成后内容使用 Mac 所选字体。

系统字形包覆盖 14、16、18、22、24、26、28、34、36 九个整数像素字号；三模式计算器补齐模式和错误提示后，当前为 3,024 个字形／1,440,876 字节，详见 `assets/generated/ui-font.json`。完整 8,430,912 字节中文 TTF 在 Mac 侧处理；固件内嵌 UI 子集和 156,880 字节拉丁字体。便签／快捷按钮仍按协议 v1 使用 18、22、28、36 四个同步字号和原有 512 KiB／512 项缓存。

Mac 默认使用随包的 HarmonyOS Sans SC Regular。升级时迁移指向旧 P4Desk 应用 Resources 的 Noto 默认路径；用户在外部目录选择的字体保留，包括文件名恰好为 `NotoSansSC-Regular.otf` 的自选字体。

## TF 完整字体与动态名称

2026-10-08：用量监控的用户名称、头像首字和详情标题改用 `Font::dynamic_font()`。此前它们走系统字形子集，没有接入 TF 字体，子集以外的中文会画成缺字框。

动态名称逐字使用当前便签字形包、文件字形包、TF 完整字体，再回退到内置 UI 子集和拉丁字体。系统标签仍以内置 UI 为优先，便签与文件仍保留各自字形包的优先级。文字测量、截断和绘制使用同一解析路径。

设备使用固定 `espressif/freetype 2.14.3~1` 的文件流读取完整 TTF／OTF，依次尝试：

1. `/sdcard/fonts/HarmonyOS_Sans_SC_Regular.ttf`
2. `/sdcard/fonts/NotoSansCJKsc-Regular.otf`
3. `/sdcard/typeface/HarmonyOS_Sans_SC_Regular.ttf`

本次已从实际 TF 目录确认第二项存在，约 16.4 MB。字体通过 Unicode cmap 查找字形，不把整份文件或全部轮廓展开进内存。FreeType 仅保留一个字体文件句柄，自定义分配器使用 PSRAM，预算 2 MiB（包含分配头和 realloc 短时重叠）；Rust alpha8 缓存限制为 512 KiB／512 项。

首次打开字体和首屏五个用户的字形预热在现有监控后台线程完成。UI 缓存未命中时只投递有界请求（最多 256 项、每批 32 项），后台完成后通知文字布局重测和 UI 重绘，不在 UI 线程执行 FreeType／TF 读取，也不新建字体线程。未支持字符及暂时读取失败可回退，不会永久缓存成缺字；同一请求有短暂重试间隔。

文件缺失、格式不支持、TF 读取异常或预算不足不会影响基础界面启动。Emoji 彩色字体不在此次加载范围，原字体未收录的字符仍会显示占位字形。字体读取状态、内存及成功次数仅记录数值，不记录用户名称、字符或正文。测试、构建、刷机及实屏确认分别记录在 `acceptance-usage-typeface.json`。

## 生成与检查

```sh
python3 scripts/generate-ui-fonts.py
python3 scripts/generate-ui-fonts.py --check
cargo test --workspace
swift test --package-path desktop/macos
./scripts/build-firmware.sh
./scripts/build-macos.sh
```

生成器验证固定源字体 SHA256，再调用 Rust `p4desk-fontpack` 烘焙和校验。`--check` 在临时目录生成并逐字节比较 `ui.p4f` 与 `ui-font.json`，不覆盖已保存资源。

`artifacts/pad-typeface` 的桌面、工具页面和前后对比来自真实 Rust UI 渲染路径，使用固定演示数据。构建、刷机、Mac 安装与实板观感分别记录在 [acceptance-pad-typeface.json](acceptance-pad-typeface.json)。

2026-09-29：Rust 94 项与 Swift 53 项测试通过，固件已刷入，Developer ID Application 签名的 Mac 应用已完整替换安装到 `/Applications/P4Desk.app` 并完成签名及随包字体工具检查。用户在实板确认“字体更舒服，效果合适”，桌面与设置页字形和粗细观感验收通过。
