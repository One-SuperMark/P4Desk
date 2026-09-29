use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const PROTOCOL_VERSION: u16 = 1;
pub const SCREEN_WIDTH: u16 = 1024;
pub const SCREEN_HEIGHT: u16 = 600;
pub const HEADER_SIZE: usize = 16;
pub const KIND_JPEG: u8 = 3;
pub const KIND_CONTROL: u8 = 16;
pub const KIND_RESOURCE: u8 = 17;
pub const CRC_PRESENT: u8 = 1;
pub const MAX_JPEG: usize = 1_048_576;
pub const MAX_CONTROL: usize = 65_536;
pub const MAX_RESOURCE: usize = 32_768 + 4;
/// Reserve space for the control envelope around a complete snapshot.
pub const MAX_SNAPSHOT_BYTES: usize = 60 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode { #[default] Pad, Display }
impl Mode { pub fn as_u32(self) -> u32 { if self == Self::Display { 1 } else { 0 } } }

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default)] pub generation: u64,
    #[serde(default)] pub notes: Vec<Note>,
    #[serde(default)] pub buttons: Vec<Button>,
    #[serde(default)] pub deleted_note_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note { pub id: String, pub title: String, pub body: String, #[serde(default)] pub updated_ms: u64 }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Button { pub id: String, pub label: String, pub action: Action }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Shortcut { key_code: u16, modifiers: u32 },
    Application { bundle_path: String },
    Media { usage: u16 },
}
impl Snapshot {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.notes.len() > 32 || self.buttons.len() > 48 || self.deleted_note_ids.len() > 4096 { return Err("item_limit"); }
        let valid_id = |id: &str| !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        let mut ids = HashSet::new();
        for n in &self.notes {
            if !valid_id(&n.id) || !ids.insert(&n.id) { return Err("note_id"); }
            if n.title.chars().count() > 128 || n.body.chars().count() > 2000 { return Err("note_length"); }
        }
        ids.clear();
        for b in &self.buttons {
            if !valid_id(&b.id) || !ids.insert(&b.id) { return Err("button_id"); }
            if b.label.chars().count() > 64 { return Err("button_label"); }
            match &b.action {
                Action::Shortcut { key_code, modifiers } if *key_code > 127 || modifiers & !0x00ff_0000 != 0 => return Err("shortcut"),
                Action::Application { bundle_path } if !bundle_path.starts_with('/') || !bundle_path.ends_with(".app") || bundle_path.contains('\0') || bundle_path.len() > 4096 => return Err("application"),
                Action::Media { usage } if ![0xb0, 0xb5, 0xb6, 0xcd, 0xe2, 0xe9, 0xea].contains(usage) => return Err("media"),
                _ => (),
            }
        }
        if self.deleted_note_ids.iter().any(|id| !valid_id(id)) { return Err("deleted_note_id"); }
        if serde_json::to_vec(self).map_err(|_| "snapshot_json")?.len() > MAX_SNAPSHOT_BYTES {
            return Err("snapshot_bytes");
        }
        Ok(())
    }
    pub fn apply_deletions(&mut self) {
        let deleted: HashSet<_> = self.deleted_note_ids.iter().collect();
        self.notes.retain(|n| !deleted.contains(&n.id));
    }
    pub fn merge_deletions(&mut self, other: &Snapshot) {
        self.deleted_note_ids.extend(other.deleted_note_ids.iter().cloned());
        self.deleted_note_ids.sort(); self.deleted_note_ids.dedup(); self.apply_deletions();
    }
    pub fn font_text(&self) -> String {
        let mut text = String::new();
        for n in &self.notes { text.push_str(&n.title); text.push('\n'); text.push_str(&n.body); text.push('\n'); }
        for b in &self.buttons { text.push_str(&b.label); text.push('\n'); }
        text
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum HostMessage {
    Hello { request_id: u16, version: u16 },
    Heartbeat { request_id: u16 },
    SetMode { request_id: u16, mode: Mode, session: u32, #[serde(default)] jpeg_rotation_degrees: u16 },
    TimeSync { request_id: u16, unix_ms: i64, timezone_minutes: i32 },
    GetState { request_id: u16 },
    SyncBegin { request_id: u16, generation: u64, state: Snapshot, font_length: u32, font_sha256: String },
    SyncCommit { request_id: u16, generation: u64 },
    SyncAbort { request_id: u16 },
}
impl HostMessage {
    pub fn request_id(&self) -> u16 {
        match self { Self::Hello { request_id, .. } | Self::Heartbeat { request_id } | Self::SetMode { request_id, .. }
            | Self::TimeSync { request_id, .. } | Self::GetState { request_id } | Self::SyncBegin { request_id, .. }
            | Self::SyncCommit { request_id, .. } | Self::SyncAbort { request_id } => *request_id }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchPoint { pub id: u8, pub x: u16, pub y: u16 }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DeviceMessage {
    Caps { request_id: u16, version: u16, width: u16, height: u16, max_jpeg: u32, max_control: u32, sd_ready: bool, mode: Mode, #[serde(default)] direct_jpeg_rotation_degrees: u16 },
    Ack { request_id: u16, acknowledged: String, ok: bool, #[serde(skip_serializing_if = "Option::is_none")] error: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] generation: Option<u64> },
    State { request_id: u16, state: Snapshot },
    Touch { session: u32, sequence: u16, points: Vec<TouchPoint>, stamp_us: u64 },
    Action { action_id: String },
    RequestMode { mode: Mode },
    RequestTimeSync,
    Status { mode: Mode, sd_ready: bool, time_valid: bool, generation: u64 },
    FramePresented { session: u32, sequence: u16, device_us: u64 },
}

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xffff;
    for &byte in data { crc ^= (byte as u16) << 8; for _ in 0..8 { crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 }; } }
    crc
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireHeader { pub crc16: u16, pub kind: u8, pub flags: u8, pub x: u16, pub y: u16, pub width: u16, pub height: u16, pub sequence: u16, pub payload_length: u32 }
impl WireHeader {
    pub fn validate(&self) -> Result<(), &'static str> {
        let n = self.payload_length as usize;
        if n == 0 || self.sequence > 1023 || self.flags & !CRC_PRESENT != 0 { return Err("header"); }
        if self.kind == KIND_JPEG {
            if self.x != 0 || self.y != 0 || self.width != SCREEN_WIDTH || self.height != SCREEN_HEIGHT || n > MAX_JPEG { return Err("jpeg_header"); }
        } else {
            if self.x != 0 || self.y != 0 || self.width != 0 || self.height != 0 || self.flags != CRC_PRESENT { return Err("control_header"); }
            match self.kind { KIND_CONTROL if n <= MAX_CONTROL => (), KIND_RESOURCE if (4..=MAX_RESOURCE).contains(&n) => (), _ => return Err("kind_length") }
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8; HEADER_SIZE]) -> Result<Self, &'static str> {
        let u16_at = |i| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        let packed = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let h = Self { crc16: u16_at(0), kind: bytes[2], flags: bytes[3], x: u16_at(4), y: u16_at(6), width: u16_at(8), height: u16_at(10), sequence: (packed & 1023) as u16, payload_length: packed >> 10 };
        h.validate()?; Ok(h)
    }
    pub fn encode(&self) -> Result<[u8; HEADER_SIZE], &'static str> {
        self.validate()?; let mut bytes = [0u8; HEADER_SIZE];
        bytes[..2].copy_from_slice(&self.crc16.to_le_bytes()); bytes[2] = self.kind; bytes[3] = self.flags;
        for (offset, value) in [(4,self.x),(6,self.y),(8,self.width),(10,self.height)] { bytes[offset..offset+2].copy_from_slice(&value.to_le_bytes()); }
        bytes[12..].copy_from_slice(&(self.payload_length << 10 | self.sequence as u32).to_le_bytes()); Ok(bytes)
    }
}
#[derive(Debug, Clone)]
pub struct Packet { pub header: WireHeader, pub payload: Vec<u8> }
impl Packet {
    pub fn new(kind: u8, sequence: u16, payload: Vec<u8>) -> Result<Self, &'static str> {
        let jpeg = kind == KIND_JPEG;
        let h = WireHeader { crc16: if jpeg { 0 } else { crc16(&payload) }, kind, flags: if jpeg { 0 } else { CRC_PRESENT },
            x:0,y:0,width:if jpeg {SCREEN_WIDTH}else{0},height:if jpeg{SCREEN_HEIGHT}else{0},sequence,payload_length:payload.len().try_into().map_err(|_| "payload_length")? };
        h.validate()?; Ok(Self { header:h, payload })
    }
    pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
        if self.payload.len() != self.header.payload_length as usize { return Err("payload_length"); }
        let mut bytes = self.header.encode()?.to_vec(); bytes.extend_from_slice(&self.payload); Ok(bytes)
    }
    pub fn control<T: Serialize>(sequence: u16, message: &T) -> Result<Self, String> {
        Self::new(KIND_CONTROL,sequence,serde_json::to_vec(message).map_err(|_| "json".to_owned())?).map_err(str::to_owned)
    }
}
#[derive(Default)]
pub struct StreamDecoder { header_bytes: Vec<u8>, header: Option<WireHeader>, payload: Vec<u8>, pub errors: u64 }
impl StreamDecoder {
    pub fn reset(&mut self) { self.header_bytes.clear(); self.header=None; self.payload.clear(); }
    pub fn feed(&mut self, mut bytes: &[u8]) -> Vec<Packet> {
        let mut ready=Vec::new();
        while !bytes.is_empty() {
            if self.header.is_none() {
                let n=(HEADER_SIZE-self.header_bytes.len()).min(bytes.len());
                self.header_bytes.extend_from_slice(&bytes[..n]); bytes=&bytes[n..];
                if self.header_bytes.len()<HEADER_SIZE { continue; }
                match WireHeader::decode(self.header_bytes.as_slice().try_into().unwrap()) {
                    Ok(h)=>{self.header=Some(h); self.payload.clear(); self.payload.reserve(h.payload_length as usize);},
                    Err(_)=>{self.errors+=1; self.header_bytes.remove(0); continue;}
                }
            }
            let h=self.header.unwrap(); let n=(h.payload_length as usize-self.payload.len()).min(bytes.len());
            self.payload.extend_from_slice(&bytes[..n]); bytes=&bytes[n..];
            if self.payload.len()==h.payload_length as usize {
                if h.flags & CRC_PRESENT != 0 && crc16(&self.payload)!=h.crc16 { self.errors+=1; self.reset(); }
                else { ready.push(Packet{header:h,payload:std::mem::take(&mut self.payload)}); self.reset(); }
            }
        }
        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn crc_known_vector(){assert_eq!(crc16(b"123456789"),0x29b1);}
    #[test] fn all_chunk_boundaries(){let p=Packet::control(1023,&HostMessage::Hello{request_id:1023,version:1}).unwrap();let data=p.encode().unwrap(); for split in 0..=data.len(){let mut d=StreamDecoder::default();let mut got=d.feed(&data[..split]);got.extend(d.feed(&data[split..]));assert_eq!(got.len(),1);assert_eq!(got[0].payload,p.payload);}}
    #[test] fn concatenated_and_single_byte(){let p=Packet::control(9,&HostMessage::Heartbeat{request_id:9}).unwrap().encode().unwrap();let both=[p.clone(),p].concat();let mut d=StreamDecoder::default();let got:Vec<_>=both.iter().flat_map(|b|d.feed(&[*b])).collect();assert_eq!(got.len(),2);assert_eq!(d.errors,0);}
    #[test] fn corrupt_crc_then_recovery(){let mut bad=Packet::control(1,&HostMessage::Heartbeat{request_id:1}).unwrap().encode().unwrap();bad[16]^=1;let good=Packet::control(2,&HostMessage::Heartbeat{request_id:2}).unwrap().encode().unwrap();let mut d=StreamDecoder::default();let got=d.feed(&[bad,good].concat());assert_eq!(got.len(),1);assert_eq!(got[0].header.sequence,2);assert_eq!(d.errors,1);}
    #[test] fn reject_oversize_and_wrong_resolution(){assert!(Packet::new(KIND_JPEG,1,vec![0;MAX_JPEG+1]).is_err());let mut h=Packet::new(KIND_JPEG,1,vec![1]).unwrap().header;h.height=576;assert!(h.encode().is_err());}
    #[test] fn deletion_wins(){let mut s=Snapshot{notes:vec![Note{id:"n1".into(),title:"便签".into(),body:"会议".into(),updated_ms:10}],..Default::default()};let other=Snapshot{deleted_note_ids:vec!["n1".into()],..Default::default()};s.merge_deletions(&other);assert!(s.notes.is_empty());assert_eq!(s.deleted_note_ids,vec!["n1"]);}
    #[test] fn malicious_ids_and_duplicates(){let n=Note{id:"../bad".into(),title:"".into(),body:"".into(),updated_ms:0};assert!(Snapshot{notes:vec![n],..Default::default()}.validate().is_err());}
    #[test] fn aggregate_snapshot_is_bounded(){let notes=(0..32).map(|i|Note{id:format!("n{i}"),title:"便签".into(),body:"中".repeat(2000),updated_ms:0}).collect();assert_eq!(Snapshot{notes,..Default::default()}.validate(),Err("snapshot_bytes"));}
    #[test] fn shared_wire_fixture(){let fixture:serde_json::Value=serde_json::from_str(include_str!("../../../tests/fixtures/hello-v1.json")).unwrap();let payload=fixture["payload_utf8"].as_str().unwrap().as_bytes().to_vec();let encoded=Packet::new(KIND_CONTROL,7,payload).unwrap().encode().unwrap();let actual:String=encoded.iter().map(|b|format!("{b:02x}")).collect();assert_eq!(actual,fixture["packet_hex"].as_str().unwrap());}

    #[test]
    fn legacy_mode_request_keeps_unrotated_jpeg_contract() {
        let message: HostMessage = serde_json::from_slice(
            br#"{"op":"set_mode","request_id":8,"mode":"display","session":42}"#,
        ).unwrap();
        assert!(matches!(message, HostMessage::SetMode {
            request_id: 8, mode: Mode::Display, session: 42, jpeg_rotation_degrees: 0,
        }));
    }

    #[test]
    fn legacy_caps_disable_direct_jpeg_rotation() {
        let message: DeviceMessage = serde_json::from_slice(
            br#"{"op":"caps","request_id":2,"version":1,"width":1024,"height":600,"max_jpeg":1048576,"max_control":65536,"sd_ready":true,"mode":"pad"}"#,
        ).unwrap();
        assert!(matches!(message, DeviceMessage::Caps {
            direct_jpeg_rotation_degrees: 0, version: 1, ..
        }));
    }

    #[test]
    fn direct_jpeg_rotation_fields_round_trip_in_v1_controls() {
        let mode = HostMessage::SetMode {
            request_id: 3, mode: Mode::Display, session: 43, jpeg_rotation_degrees: 180,
        };
        let packet = Packet::control(3, &mode).unwrap();
        let mut stream = StreamDecoder::default();
        let received = stream.feed(&packet.encode().unwrap());
        assert_eq!(received.len(), 1);
        assert!(matches!(serde_json::from_slice::<HostMessage>(&received[0].payload).unwrap(),
            HostMessage::SetMode { jpeg_rotation_degrees: 180, .. }));
        assert_eq!(stream.errors, 0);

        let caps = DeviceMessage::Caps {
            request_id: 2, version: PROTOCOL_VERSION, width: SCREEN_WIDTH, height: SCREEN_HEIGHT,
            max_jpeg: MAX_JPEG as u32, max_control: MAX_CONTROL as u32,
            sd_ready: true, mode: Mode::Pad, direct_jpeg_rotation_degrees: 180,
        };
        let received: DeviceMessage = serde_json::from_slice(&serde_json::to_vec(&caps).unwrap()).unwrap();
        assert!(matches!(received, DeviceMessage::Caps {
            direct_jpeg_rotation_degrees: 180, version: 1, ..
        }));
    }
}
