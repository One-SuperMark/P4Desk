# JPEG 全范围 BT.601 修正

本组件在构建目录生成固定 ESP-IDF **6.0.2** `jpeg_decode.c` 的项目副本。全局 SDK 保持只读；生成文件保留原 basename、Apache-2.0 声明和解码器生命周期。组件没有新增产品协议或另一套解码器。

## 颜色契约与适用范围

[JFIF 1.02](https://www.w3.org/Graphics/JPEG/jfif3.pdf) 的 8 bit Y/Cb/Cr 使用完整的 0…255 码值。其逆变换为：

```text
R = Y + 1.402   × (Cr − 128)
G = Y − 0.34414 × (Cb − 128) − 0.71414 × (Cr − 128)
B = Y + 1.772   × (Cb − 128)
```

IDF 6.0.2 `esp_hal_dma/include/hal/dma2d_types.h` 中的 BT.601/709 表使用视频有限范围：Y 为 16…235，Cb/Cr 为 16…240。该版本 JPEG 驱动的 BT.601 RGB 输出直接使用此表。主机以 sRGB 输入生成全范围 BT.601 JPEG 时，两端须采用相同范围。

本组件**仅**覆盖 JPEG BT.601 解码的 RGB565/RGB888 转换系数，保留原 endian、GRAY/YUV 输出、BT.709 路径和其他 DMA 用户的配置。当前 RGB565 的 `JPEG_DEC_RGB_ELEMENT_ORDER_BGR` 对应小端字节顺序；改成 RGB 会交换两个字节，不是本修正的一部分。

微雪本地 `12_usb_extend_screen/main/app_lcd_p4.c` 同样配置 RGB565、BGR，并省略 `conv_std`（默认 BT.601）。其 BSP 3.0.1 在 IDF 6 上同样配置 DSI RGB565 输入/输出。因此本修正针对解码转换范围，不调整面板、触摸方向或面板格式。

## Q8 系数

硬件计算 `256 × Q = A × Y + B × Cb + C × Cr + D`：

| 输出 | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| R | 256 | 0 | 359 | −45952 |
| G | 256 | −88 | −183 | 34688 |
| B | 256 | 454 | 0 | −58112 |

偏移是所用整数系数乘以 128，所以全部中性灰 `(Y,128,128)` 精确回到 `(Y,Y,Y)`。相对上述 JFIF 浮点公式，未裁剪 8 bit 通道的最大系数量化误差为 R 0.044、G 0.140、B 0.184 码值；硬件的最终取整及 RGB565 量化另计。寄存器 A/B/C/D 的有符号位宽分别为 10/11/10/18，负值以对应位宽的两补码写入。局部寄存器结构先清零，保留位也为零。

## DMA 所有权与生成检查

原驱动申请 pool 0，JPEG 事务要求 `DMA2D_CHANNEL_FUNCTION_FLAG_RX_REORDER`。固定 SDK 的 RX reorder/CSC mask 都只有 bit 0；调度器已为 `on_job_picked` 取得 RX0 独占权。补丁在原 `dma2d_configure_color_space_conversion()` **之后、DMA start 之前**写入六个参数寄存器。此时调度器已经释放 group spinlock；安全性来自通道独占，补丁不申请额外锁、不分配、不复制像素。下个 PPA/JPEG 等事务仍按原逻辑配置自己的转换矩阵。

生成器检查原文件 SHA256、唯一替换锚点、pool 0、RX reorder、一个 TX/一个 RX、configure-before-start 完整片段；另验证实际 `esp_driver_dma/src/dma2d.c` 的 SHA256，因为 RX0 独占推论依赖它的 mask 筛选和通道取得逻辑。生成 C 另检查 DMA 实例数、RX mask 和参数结构大小。任一不匹配均停止；CMake 必须恰好替换 `__idf_esp_driver_jpeg` 的一个原 source，并加入原组件目录作为私有 include。CLI 拒绝在全局 SDK 任意子目录内写入生成文件或 manifest。

固定原文件 SHA256：

```text
fa4ce744bfaea32e9becbe93db14f65f8f9a4386e637dc83d031f7c3d194c488
```

必要的 DMA2D 调度源 SHA256：

```text
510355c193cc5c651e3b9158c1c38055b7a2e7a7df4d189e55530a9e54003f26
```

构建目录的 `generated/patch-manifest.json` 记录来源/生成 SHA、系数、作用域和验证状态。

## 验证与当前限制

局部回归：

```sh
PYTHONDONTWRITEBYTECODE=1 python3 firmware/components/jpeg_full_range/tests/test_full_range_patch.py \
  --idf-path /Volumes/work/esp/esp-idf-v6.0.2
```

测试覆盖真实 JFIF 公式的系数误差、中性灰全部码值、寄存器符号位宽、生成 C 的范围条件/六次寄存器写入、SHA/唯一锚点/所有权片段漂移拒绝和 CMake source 置换。

**实机颜色验证尚未完成。** SDK 6.0.2 解码测试只验证输出大小、错误处理、性能和内存，未提供颜色参考图逐像素断言；源码中未找到解码器内部的 full→limited 缩放配置，但这不能独立证明硅片不存在隐含处理。主机合成 JPEG 的软件结果也不能替代 P4 硬件验证。后续用自制灰阶、橙色、RGB 原色和细文字比较修正前后，检查黑白端点、暗部灰阶、颜色偏差、采样格式和 copy=0 路径；日志仅记录误差、耗时和 pass/fail，不记录画面内容。显示清晰度还受主机采样与 JPEG 压缩影响。

## 许可证

生成的驱动源继承 Espressif 的 Apache-2.0 声明。生成器、构建配置和项目测试使用本项目 MIT 许可证。
