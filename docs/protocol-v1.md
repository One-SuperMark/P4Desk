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

JPEG 最大 1 MiB（HELLO 协商），完整 baseline JPEG。视频不要求应用层CRC，控制和资源使用CRC。CONTROL为UTF-8 JSON，最大64KiB。RESOURCE为u32 offset加原始字体包字节，单包数据不超过32768字节，使用帧头sequence作为请求ID。接收器支持任意USB短包/拼包；无效头或长度重置当前消息，超时/断线清空流。主机一次完整发送一个消息，控制优先于下一待发JPEG，禁止在JPEG payload中插入控制数据。

## JSON 控制

使用 `op` 作 discriminator，snake_case。请求含 `request_id` (0..1023)，ACK使用相同ID；主机避免复用在途ID。mode只有 `pad` 与 `display`。副屏session为u32，连接或重新进入副屏时重新分配。

主机到设备：

- `hello {request_id, version:1}`
- `heartbeat {request_id}`，1秒一次，设备3秒未收到回Pad
- `set_mode {request_id, mode, session}`
- `time_sync {request_id, unix_ms:i64, timezone_minutes:i32}`：UTC 毫秒范围 `946684800000..=4102444800000`（2000-01-01 至 2100-01-01，含端点），时区分钟 `-840..=840`；越界 ACK `ok=false, error="time_invalid"`，时钟与时区不变。
- `get_state {request_id}`
- `sync_begin {request_id, generation:u64, state:Snapshot, font_length:u32, font_sha256:string}`
- RESOURCE offset+chunk，每个包等待ACK后发送下一包
- `sync_commit {request_id, generation}`
- `sync_abort {request_id}`

设备到主机：

- `caps {request_id, version:1, width:1024, height:600, max_jpeg:1048576, max_control:65536, sd_ready:bool, mode}`
- `ack {request_id, acknowledged:string, ok:bool, error?:string, generation?:u64}`
- `state {request_id, state:Snapshot}`（本地删除时request_id=0，主机合并删除记录）
- `touch {session:u32, sequence:u16, points:[{id:u8,x:u16,y:u16}], stamp_us:u64}`；空points表示全部抬起。JSON 的完整 u16 序号按 65536 回绕，wire 帧头仅取低 10 位；优先 release 到达后，Mac 拒绝排队的旧 touch。
- `action {action_id:string}`；只发送ID，主机按设备已确认的按钮代次查实际动作
- `request_mode {mode}`；板上请求进入/退出副屏
- `request_time_sync {}`；板上 USB 校时按钮请求主机立即发送当前时间
- `status {mode, sd_ready:bool, time_valid:bool, generation:u64}`
- `frame_presented {session:u32, sequence:u16, device_us:u64}`；在LCD呈现事件后发送

## Snapshot

`{generation:u64, notes:[Note], buttons:[Button], deleted_note_ids:[string]}`。

Note=`{id:string,title:string,body:string,updated_ms:u64}`。

Button=`{id:string,label:string,action:Action}`。

Action由kind区分：`shortcut {key_code:u16,modifiers:u32}`（CGEventFlags原始位值）、`application {bundle_path:string}`、`media {usage:u16}`（HID Consumer usage）。首版最多32条便签、48个按钮；每条便签title<=128 Unicode字符，body<=2000字符，按钮label<=64字符，所有ID<=64 ASCII字节。Snapshot 编码为 UTF-8 JSON 后总计不超过 60 KiB，给控制消息外层字段预留空间；超过时拒绝同步并保留当前有效数据。删除记录优先于旧内容，只有显式创建新ID恢复便签。

SYNC把Snapshot与单个动态字体包组成同一代。TF目录 `/sdcard/p4desk/generations/<generation>/`。新代完整写入并校验SHA256和文字覆盖后提交，保留上一有效代，启动选最新完整代。挂载或校验失败不格式化、不切换；错误字符串只含操作原因，不含正文。

## C/Rust HAL

`include/p4desk_hal.h` 是固件唯一FFI定义。C完成DSI/GT911/TF/USB初始化后调用Rust入口。Rust负责JSON、快照持久化、字体与UI。C负责JPEG解码、唯一面板owner、触点采样、心跳回退与USB收发。

原始触摸快照 FFI 使用 `p4desk_touch_point_t`（8字节）与 `p4desk_touch_frame_t`（56字节，points偏移16），两侧检查结构大小和偏移；通过 `p4desk_get_raw_touch` 复制 GT911 真实触点 ID／时间／session。它与 Pad 的稳定主触点 `host_touch_get_point` 独立。USB 数据不直接传输 C 结构，仍按上述 JSON 和小端帧头编码。

Rust的 begin/end frame 成对调用，flush指针只在同步调用期间有效；C立即复制到受锁保护的Pad canonical framebuffer。面板完成事件决定显示缓冲回收。切模式清空旧帧并更改显示epoch，防止迟到任务输出旧画面。Pad只接收固定首触点，副屏回传完整触点并识别三指长按1秒退出。
