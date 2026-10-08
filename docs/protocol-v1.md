# P4 Desk USB 协议 v1

设备: ESP32-P4-WIFI6-Touch-LCD-7B，1024×600 RGB565。USB HS vendor interface 0，OUT 0x01，IN 0x81；媒体操作使用独立标准 HID Consumer。

## 消息封装

沿用上游 16 字节帧头，所有整数显式 little endian，禁止跨语言拷贝 C 位域：

| 偏移 | 字段 | 大小 |
| --- | --- | --- |
| 0 | payload CRC16-CCITT-FALSE，flags bit0=1 时校验；init=0xffff, poly=0x1021 | 2 |
| 2 | kind: JPEG=3, CONTROL=16, RESOURCE=17 | 1 |
| 3 | flags: CRC_PRESENT=1，其余保留为0 | 1 |
| 4/6 | x/y，完整JPEG均为0，其余消息为0 | 2+2 |
| 8/10 | width/height，JPEG为1024/600，其余为0 | 2+2 |
| 12 | u32，低10位 sequence，高22位 payload length | 4 |

JPEG 最大 1 MiB（HELLO 协商），完整 baseline JPEG。视频不要求应用层CRC，控制和资源使用CRC。CONTROL为UTF-8 JSON，最大64KiB。RESOURCE为u32 offset加原始资源字节（便签字体包或有效文件上传中的文件块），单包数据不超过32768字节，使用帧头sequence作为请求ID。接收器支持任意USB短包/拼包；无效头或长度重置当前消息，超时/断线清空流。主机一次完整发送一个消息，控制优先于下一待发JPEG，禁止在JPEG payload中插入控制数据。

JPEG 图像颜色采用 sRGB 输入、JFIF 的 **完整范围 YCbCr／BT.601 系数**（Y 与色度使用 0–255）。支持 4:4:4、4:2:2、4:2:0 和灰度；不可将没有矩阵标记的 BT.709 输出直接当作 JFIF 发送。Mac 清晰度方案优先完整色度，实际编码结果必须仍为 SOF0、8 bit、1024×600；超过 1 MiB 时在有限质量梯度内重新编码，最终仍越界则丢弃该帧，不截断 JPEG。

板上 `jpeg_full_range` 使用固定 ESP-IDF 6.0.2 的项目内生成源，仅在 JPEG 事务的 RGB565／RGB888 BT.601 输出配置阶段替换为 JFIF 完整范围矩阵。原 BGR 小端 RGB565、灰度和 BT.709 路径保持各自格式。转换仍由硬件执行，Pad 的原生 RGB565 画布不经过该矩阵。

## JSON 控制

使用 `op` 作 discriminator，snake_case。请求含 `request_id` (0..1023)，ACK使用相同ID；主机避免复用在途ID。mode只有 `pad` 与 `display`。副屏session为u32，连接或重新进入副屏时重新分配。

主机到设备：

- `hello {request_id, version:1}`
- `heartbeat {request_id}`，1秒一次，设备3秒未收到回Pad
- `set_mode {request_id, mode, session, jpeg_rotation_degrees?:u16}`；缺省为0，表示发送原方向的 JPEG。仅当设备 `caps.direct_jpeg_rotation_degrees=180` 时，Mac 可请求180并在编码前旋转可见的1024×600实际像素，设备直接解码到 LCD 缓冲。Pad 必须为0；不支持的非零角度返回 `jpeg_rotation`，不改变模式。角度变化与 session／模式变化一样递增 epoch，丢弃旧 JPEG。
- `time_sync {request_id, unix_ms:i64, timezone_minutes:i32}`：UTC 毫秒范围 `946684800000..=4102444800000`（2000-01-01 至 2100-01-01，含端点），时区分钟 `-840..=840`；越界 ACK `ok=false, error="time_invalid"`，时钟与时区不变。
- `get_state {request_id}`
- `sync_begin {request_id, generation:u64, state:Snapshot, font_length:u32, font_sha256:string}`
- RESOURCE offset+chunk，每个包等待ACK后发送下一包
- `sync_commit {request_id, generation}`
- `sync_abort {request_id}`

设备到主机：

- `caps {request_id, version:1, width:1024, height:600, max_jpeg:1048576, max_control:65536, sd_ready:bool, mode, direct_jpeg_rotation_degrees?:u16}`；缺省／0表示不启用主机旋转协商，180表示支持预旋转后直接解码。新增字段保持v1兼容，旧主机忽略。
- `ack {request_id, acknowledged:string, ok:bool, error?:string, generation?:u64}`
- `state {request_id, state:Snapshot}`（本地删除时request_id=0，主机合并删除记录）
- `touch {session:u32, sequence:u16, points:[{id:u8,x:u16,y:u16}], stamp_us:u64}`；空points表示全部抬起。JSON 的完整 u16 序号按 65536 回绕，wire 帧头仅取低 10 位；优先 release 到达后，Mac 拒绝排队的旧 touch。
- `action {action_id:string}`；只发送ID，主机按设备已确认的按钮代次查实际动作
- `request_mode {mode}`；板上请求进入/退出副屏
- `request_time_sync {}`；板上 USB 校时按钮请求主机立即发送当前时间
- `status {mode, sd_ready:bool, time_valid:bool, generation:u64}`
- `frame_presented {session:u32, sequence:u16, device_us:u64}`；新版固件在该帧开始扫描后，首次完整 LCD DMA 源读取完成时发送一次。可选性能元数据为 `jpeg_bytes:u32`、`decode_us:u64`、`copy_us:u64`、`present_us:u64`：依次为该帧 JPEG payload 字节数、头校验及硬件解码耗时、可见 600 行裁剪／旋转拷贝耗时、LCD 提交开始至该帧完整 DMA 读取完成耗时。协商直接解码后 `copy_us=0`。旧固件的 `present_us` 包含两次刷新安全等待。时间均来自板端单调时钟；旧主机忽略新增字段，新主机接受缺少字段的旧固件。DMA 完成用于保证缓冲所有权，不等于光学显示完成；回执到达 Mac 还包含 USB 回传时间。

## 可选文件传输（v1 兼容扩展）

`caps.file_transfer:bool` 缺省为 false，新固件工作线程就绪时为 true。文件传输只允许 Pad 模式且 TF 就绪，与便签字体同步、副屏及待进入副屏互斥。路径均相对 `/sdcard`（空字符串为根），使用 docs/file-manager.md 的 FAT 和系统目录限制。

| 请求 | 字段 | 成功响应 |
| --- | --- | --- |
| `file_list` | request_id, path:string, offset:u32, limit:u16（1～32） | `file_listing` |
| `file_mkdir` | request_id, path:string | ACK |
| `file_upload_begin` | request_id, path:string, length:u64（0～256 MiB）, sha256:64 hex | ACK |
| RESOURCE（kind 17） | offset:u32 小端 + 1～32768 原始文件字节 | ACK acknowledged=`file_chunk` |
| `file_upload_commit` | request_id | 完成 TF 重读校验及提交后 ACK |
| `file_upload_abort` | request_id | 清理未提交文件后 ACK |
| `file_font_install` | request_id, path:string, sha256:64 hex | 校验独立 P4F1 字库及保存指针后 ACK |

`file_listing`：`{request_id,path,entries:[{name,path,directory:bool,size:u64,modified_seconds:i64|null,read_only:bool}],total:u32,truncated:bool,read_only:bool}`。每页最多 32 项；设备当前目录结果集最多 256 项，truncated 同时提示截断。普通 JSON 最大仍为 64 KiB。

ACK 使用原格式，`acknowledged` 为请求 op。上传 ACK 表示该块已写入或整个文件已提交，不表示仅入队。每块等待 ACK，偏移必须等于已接收字节数；长度为 0 时 begin 后直接 commit。RESOURCE 有效上传期间解释为文件块，否则保持便签字体块。单次只允许一个上传，严禁两类传输交叉。

校验使用流式 SHA256，先验证接收数据，再在关闭与同步后从 TF 卡重读。仅完整正确的新文件提交；不覆盖已有文件。15 秒无后续块取消上传；commit 重读窗口为 `30秒 + ceil(length/512KiB)秒`，最大 542 秒。断线／主动取消／新 hello 握手通过 epoch 和取消 token 中止旧校验与发布，清空旧回复，避免 Mac 进程重启后沿用旧上传。

独立字库最大 8 MiB、16,384 条记录，先验证头部和索引边界再分配。P4 仅加载索引和缓存，文件名／预览 provider 不替换便签或系统 provider。安全错误码固定为 `storage_unavailable`、`files_unavailable`、`busy`、`display_busy`、`sync_busy`、`invalid_path`、`protected`、`already_exists`、`too_large`、`no_space`、`integrity`、`offset`、`cancelled` 等，不含路径或内容。未 hello 时保持 `hello_required`。

## Snapshot

`{generation:u64, notes:[Note], buttons:[Button], deleted_note_ids:[string]}`。

Note=`{id:string,title:string,body:string,updated_ms:u64}`。

Button=`{id:string,label:string,action:Action}`。

Action由kind区分：`shortcut {key_code:u16,modifiers:u32}`（CGEventFlags原始位值）、`application {bundle_path:string}`、`media {usage:u16}`（HID Consumer usage）。首版最多32条便签、48个按钮；每条便签title<=128 Unicode字符，body<=2000字符，按钮label<=64字符，所有ID<=64 ASCII字节。Snapshot 编码为 UTF-8 JSON 后总计不超过 60 KiB，给控制消息外层字段预留空间；超过时拒绝同步并保留当前有效数据。删除记录优先于旧内容，只有显式创建新ID恢复便签。

SYNC把Snapshot与单个动态字体包组成同一代。TF目录 `/sdcard/p4desk/generations/<generation>/`。新代完整写入并校验SHA256和文字覆盖后提交，保留上一有效代，启动选最新完整代。挂载或校验失败不格式化、不切换；错误字符串只含操作原因，不含正文。

## C/Rust HAL

`include/p4desk_hal.h` 是固件唯一FFI定义。C完成DSI/GT911/TF/USB初始化后调用Rust入口。Rust负责JSON、快照持久化、字体与UI。C负责JPEG解码、唯一面板owner、触点采样、心跳回退与USB收发。

原始触摸快照 FFI 使用 `p4desk_touch_point_t`（8字节）与 `p4desk_touch_frame_t`（56字节，points偏移16），两侧检查结构大小和偏移；通过 `p4desk_get_raw_touch` 复制 GT911 真实触点 ID／时间／session。

Pad 稳定主触点通过 `p4desk_poll_pad_touch` 顺序读取采样事件。`p4desk_pad_touch_event_t` 为 12 字节（`kind:u32`，`x:i32` 偏移4，`y:i32` 偏移8），C／Rust 均有编译期布局断言；kind 为 Down=1、Move=2、Up=3、Cancel=4，坐标已经转换到 Pad 逻辑坐标。64 项有界队列保留短点击和拖动历史；溢出只送 Cancel，并等所有接触释放后重新接受手势。模式改变时清空事件。Rust 每次 UI step 最多消费一项，在应用切换后的重建之前不消费下一项。`host_touch_get_point` 保留兼容的最新状态查询，当前 Pad Rust 输入路径使用事件队列。

以上为同一固件内的原生 C ABI，USB 数据不直接传输 C 结构，仍按上述 JSON 和显式小端帧头编码；USB 多点回传协议未改变。

主机预旋转仅改变副屏 JPEG 的像素方向，触摸仍使用原桌面坐标，Pad 继续使用已确认的180°软件旋转。新版 LCD driver 在项目构建目录生成固定 SDK 扩展，为每缓冲额外预留8行 JPEG MCU padding；扫描、可见几何和 USB 帧头仍为1024×600。只有明确的 FREE/BUILDING 缓冲可以写入，实际 DMA completed/next 指针及严格相邻计数决定回收；重复扫描不释放缓冲，丢事件或超时停止重用。

Rust的 begin/end frame 成对调用，flush指针只在同步调用期间有效；C立即复制到受锁保护的Pad canonical framebuffer。面板完成事件决定显示缓冲回收。切模式清空旧帧并更改显示epoch，防止迟到任务输出旧画面。Pad只接收固定首触点，副屏回传完整触点并识别三指长按1秒退出。

### 桌面 USB 入口与首帧动画（设备内部，不增加 wire 字段）

桌面图标展开至全屏主题色后，设备发送原有 `request_mode: display`。主机创建虚拟屏、捕获首张 JPEG 后沿用 `set_mode` 与视频包。仅从该桌面入口启动时，设备将收缩动画绑定至本次 epoch，成功解码首张 JPEG 才开始 600 ms 向中心收缩；Mac 直接开启副屏无此过渡。

过渡期间真实帧仍在 LCD 完成后按原格式返回 `frame_presented`（该帧可能带主题色遮罩），不把动画内部重放的相同 JPEG 当作新收到的帧重复确认。最终无覆盖帧完成 LCD DMA 后，解除输入拦截，待手指全部释放再发送新触摸。主机无需新消息或新能力位。等待超过 10 秒、断线或失败时取消待进入请求并请求 Pad，收起至准备页显示错误；未连接时照常播放开屏动画并停在准备页。

### Pad 电池读数（内部 HAL）

`int32_t p4desk_battery_voltage_mv(void)` 返回校准后的 BAT 毫伏值，无法读取返回 `-1`，C／Rust 均使用固定 32 位有符号整数。UI 每两秒读取一次后台缓存，百分比在 Rust 端平滑估算。7B 没有接入 MCU 的充电状态信号。按用户约定，`bool p4desk_typec_host_connected(void)` 返回原生 Type-C 的主机 SOF 连接状态，HAL 映射为 `PluggedInAssumed`／Unknown，独立于 Type-A 副屏 USB。此值不是 VBUS 或充电电流检测，不覆盖充电器／CH343 Type-C。

`uint32_t p4desk_reset_reason(void)` 返回固定 SDK 6.0.2 的 `esp_reset_reason_t` 数值，Rust 显示启动原因；C 静态检查 POWERON=1、BROWNOUT=9、USB=11、PWR_GLITCH=14 和 CPU_LOCKUP=15。POWERON 不能区分断电和 EN 引脚复位。本次没有新增 USB wire 字段或改变版本。

## 可选扩展：独立 Wi-Fi 用量监控

继续使用 v1 control 帧（CRC、长度和序号规则不变，头字段保持小端）。旧固件会拒绝未知 op，主机提示需要更新固件。

| Host op | 字段 | 回复与含义 |
| --- | --- | --- |
| `monitor_configure` | `request_id`, `site`, `key` | `ack` 表示已接受验证任务，**不代表保存成功** |
| `monitor_forget` | `request_id` | `ack` 表示已接受清除任务 |
| `monitor_get_status` | `request_id` | `monitor_status`，包含 `configured`, `busy`, `configuration_result`, `message` |

`configuration_result`：任务开始时为 null；设备完成验证和存储读回后为 true，失败为 false。Mac 在收到接受 ACK 后轮询此字段，不能仅依赖 `configured`（可能属于上一个配置）或 `busy`（后续统计刷新也会设置它）。一次只接受一个配置任务；清除可取消待完成配置，最终以清除记录为准。

配置不会混入便签 Snapshot、字体包或普通会话记录。响应不回传密钥；调试格式也屏蔽密钥。USB 断开不停止已配置的独立 Wi-Fi 监控。
