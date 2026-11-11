//! MCHOSE K99 V2 keyboard over the 2.4G receiver (`41e4:2001`, vendor report `0x13`).
//!
//! The receiver tunnels the same configuration blocks as the wired K99 V2
//! (feature report `0x06`, 519 bytes) in 19-byte packets with a 14-byte chunk.
//! The packet framing and block layouts were confirmed against a real K99 V2
//! on macOS; the connected receiver reports the K99 V2 signature
//! `03 00 00 00 01 DF`.

use crate::error::DeviceError;
use crate::model::{
    Battery, Capabilities, ConnectionKind, DeviceInfo, DeviceKind, DeviceSummary, LiveStatus,
};
use hidapi::{HidApi, HidDevice};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const VID: u16 = 0x41E4;
pub const PID: u16 = 0x2001;
pub const REPORT_ID: u8 = 0x13;
pub const PACKET_LEN: usize = 19;
pub const CHUNK_LEN: usize = 14;
pub const KEY_COUNT: usize = 113;
pub const KEY_LAYER_LEN: usize = KEY_COUNT * 4;
pub const PERF_LEN: usize = 128;
pub const LIGHT_LEN: usize = 490;
pub const K99_V2_SIGNATURE: [u8; 6] = [0x03, 0x00, 0x00, 0x00, 0x01, 0xDF];
const READ_TIMEOUT: Duration = Duration::from_millis(4500);
const WRITE_GAP: Duration = Duration::from_millis(1200);

pub fn summary_key(serial: Option<&str>) -> String {
    format!("kb:{VID:04x}:{PID:04x}:{}", serial.unwrap_or(""))
}

pub fn scan() -> Result<Vec<DeviceSummary>, DeviceError> {
    let api = HidApi::new()?;
    let mut out = Vec::new();
    for d in api.device_list() {
        if d.vendor_id() != VID || d.product_id() != PID || d.usage_page() != 0xFF02 {
            continue;
        }
        let key = summary_key(d.serial_number());
        if out.iter().any(|s: &DeviceSummary| s.key == key) {
            continue;
        }
        out.push(DeviceSummary {
            key,
            kind: DeviceKind::Keyboard,
            name: d.product_string().unwrap_or("MCHOSE 2.4G 键盘").to_string(),
            model: None,
            serial: d.serial_number().map(str::to_string),
            vendor_id: VID,
            product_id: PID,
            connection: ConnectionKind::Wireless,
            supported: true,
            status_note: Some("2.4G 接收器；连接后会读取键盘型号".to_string()),
            is_demo: false,
        });
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LightModeInfo {
    pub id: u8,
    pub name: String,
    pub brightness: bool,
    pub speed: bool,
    pub color: bool,
}

pub fn light_modes() -> Vec<LightModeInfo> {
    // (id, 名称, 亮度禁用, 速度禁用, 单色禁用)
    const ITEMS: &[(u8, &str, bool, bool, bool)] = &[
        (0, "关闭", true, true, true),
        (1, "常亮", false, true, false),
        (2, "呼吸", false, false, false),
        (3, "梦幻彩虹", false, false, true),
        (4, "一触即发", false, false, false),
        (5, "雨中漫步", false, false, false),
        (7, "涟漪扩散", false, false, false),
        (8, "繁星点点", false, false, false),
        (10, "川流不息", false, false, false),
        (11, "随波逐流", false, false, false),
        (12, "如影随形", false, false, false),
        (13, "正弦光波", false, false, false),
        (15, "旋转风车", true, true, true),
        (16, "七彩瀑布", false, false, true),
        (17, "花开富贵", true, true, true),
        (21, "自定义（逐键调色）", true, true, false),
    ];
    ITEMS
        .iter()
        .map(
            |(id, name, brightness_off, speed_off, color_off)| LightModeInfo {
                id: *id,
                name: (*name).to_string(),
                brightness: !brightness_off,
                speed: !speed_off,
                color: !color_off,
            },
        )
        .collect()
}

pub fn mode_has_brightness(mode: u8) -> bool {
    light_modes().iter().any(|m| m.id == mode && m.brightness)
}
pub fn mode_has_speed(mode: u8) -> bool {
    light_modes().iter().any(|m| m.id == mode && m.speed)
}
pub fn mode_has_color(mode: u8) -> bool {
    light_modes().iter().any(|m| m.id == mode && m.color)
}

fn brightness_offset(mode: u8) -> Option<usize> {
    match mode {
        1..=5 => Some(58 + (mode as usize - 1) * 2),
        7 | 8 => Some(70 + (mode as usize - 7) * 2),
        10..=13 => Some(76 + (mode as usize - 10) * 2),
        15 => Some(84),
        16 | 17 => Some(88 + (mode as usize - 16) * 2),
        _ => None,
    }
}

fn color_offset(mode: u8) -> Option<usize> {
    match mode {
        1 => Some(21),
        2 => Some(42),
        3 => Some(63),
        4 => Some(84),
        5 => Some(105),
        7 => Some(147),
        8 => Some(168),
        10 => Some(210),
        11 => Some(231),
        12 => Some(252),
        13 => Some(273),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardPerformance {
    pub rate_hz: u32,
    pub sleep_minutes: u8,
    /// speed / standard / special
    pub latency: String,
    pub win_key_locked: bool,
    pub mac_mode: bool,
    pub light_type: u8,
    pub light_mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub multi_color: bool,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardSnapshot {
    pub info: DeviceInfo,
    pub online: bool,
    pub battery: Option<Battery>,
    pub performance: KeyboardPerformance,
    /// 3 层 × 113 键 × 4 字节
    pub layers: Vec<Vec<[u8; 4]>>,
    pub light_modes: Vec<LightModeInfo>,
    pub read_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardInput {
    pub performance: KeyboardPerformance,
    pub layers: Vec<Vec<[u8; 4]>>,
}

struct Link {
    dev: HidDevice,
    last_write: Option<Instant>,
}

impl Link {
    fn drain(&self) {
        let mut buf = [0u8; 256];
        while let Ok(n) = self.dev.read_timeout(&mut buf, 0) {
            if n == 0 {
                break;
            }
        }
    }

    fn send_packet(&self, packet: &[u8; PACKET_LEN]) -> Result<(), DeviceError> {
        let mut out = [0u8; PACKET_LEN + 1];
        out[0] = REPORT_ID;
        out[1..].copy_from_slice(packet);
        self.dev
            .write(&out)
            .map_err(|e| DeviceError::hid(format!("键盘写入失败：{e}")))?;
        Ok(())
    }

    fn read_packet(&self, timeout: Duration) -> Option<[u8; PACKET_LEN]> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let mut buf = [0u8; 256];
            let remaining = (deadline - Instant::now()).as_millis().max(1) as i32;
            let n = self.dev.read_timeout(&mut buf, remaining).ok()?;
            if n == 0 {
                return None;
            }
            let offset = if buf[0] == REPORT_ID { 1 } else { 0 };
            if n >= offset + PACKET_LEN {
                let mut packet = [0u8; PACKET_LEN];
                packet.copy_from_slice(&buf[offset..offset + PACKET_LEN]);
                return Some(packet);
            }
        }
        None
    }

    fn read_data(&self, cmd: u8, layer: u8, min_len: usize) -> Result<Vec<u8>, DeviceError> {
        let mut last = DeviceError::timeout(format!("键盘命令 0x{cmd:02X} 超时"));
        for _ in 0..3 {
            self.drain();
            let packet = read_request(cmd, layer);
            self.send_packet(&packet)?;
            let mut packets: [Option<Vec<u8>>; 256] = std::array::from_fn(|_| None);
            let mut total = 0usize;
            let deadline = Instant::now() + READ_TIMEOUT;
            while Instant::now() < deadline {
                let Some(p) = self.read_packet(Duration::from_millis(700)) else {
                    break;
                };
                if p[0] != cmd {
                    continue;
                }
                let count = p[1] as usize;
                let seq = p[2] as usize;
                if count == 0 || seq >= count {
                    last = DeviceError::protocol("键盘无线分包编号无效");
                    break;
                }
                if total != 0 && total != count {
                    last = DeviceError::protocol("键盘无线分包总数变化");
                    break;
                }
                total = count;
                packets[seq] = Some(p[4..18].to_vec());
                if (0..total).all(|i| packets[i].is_some()) {
                    let mut data = Vec::with_capacity(total * CHUNK_LEN);
                    for item in packets[..total].iter() {
                        data.extend_from_slice(item.as_ref().unwrap());
                    }
                    if data.len() < min_len {
                        last = DeviceError::protocol("键盘无线回包不完整");
                        break;
                    }
                    return Ok(data);
                }
            }
            std::thread::sleep(Duration::from_millis(120));
        }
        Err(last)
    }

    fn write_data(&mut self, cmd: u8, data: &[u8], layer: u8) -> Result<(), DeviceError> {
        if let Some(last) = self.last_write {
            let elapsed = last.elapsed();
            if elapsed < WRITE_GAP {
                std::thread::sleep(WRITE_GAP - elapsed);
            }
        }
        self.last_write = Some(Instant::now());

        let packets = write_packets(cmd, data, layer)?;
        for packet in &packets {
            let mut acked = false;
            for attempt in 0..3 {
                if attempt > 0 {
                    std::thread::sleep(Duration::from_millis(80));
                }
                self.drain();
                self.send_packet(packet)?;
                let deadline = Instant::now() + Duration::from_secs(5);
                while Instant::now() < deadline {
                    let Some(p) = self.read_packet(Duration::from_millis(500)) else {
                        break;
                    };
                    if p[0] == cmd && p[2] == packet[2] {
                        acked = true;
                        break;
                    }
                }
                if acked {
                    break;
                }
            }
            if !acked {
                return Err(DeviceError::timeout(format!(
                    "键盘分包 {}/{} 没有确认，配置可能只写入了一部分",
                    packet[2] as usize + 1,
                    packets.len()
                )));
            }
        }
        Ok(())
    }
}

pub fn checksum(packet: &[u8; PACKET_LEN]) -> u8 {
    let mut sum = REPORT_ID as u32;
    for b in &packet[..PACKET_LEN - 1] {
        sum += *b as u32;
    }
    (sum & 0xFF) as u8
}

pub fn read_request(cmd: u8, layer: u8) -> [u8; PACKET_LEN] {
    let mut p = [0u8; PACKET_LEN];
    p[0] = cmd;
    p[1] = 1;
    if cmd == 0x41 {
        p[3] = layer << 4;
    }
    p[PACKET_LEN - 1] = checksum(&p);
    p
}

pub fn write_packets(
    cmd: u8,
    data: &[u8],
    layer: u8,
) -> Result<Vec<[u8; PACKET_LEN]>, DeviceError> {
    if data.is_empty() {
        return Err(DeviceError::invalid("键盘写入数据为空"));
    }
    let total = data.len().div_ceil(CHUNK_LEN);
    if total > 255 {
        return Err(DeviceError::invalid("键盘写入超过 255 个分包"));
    }
    let mut out = Vec::with_capacity(total);
    for seq in 0..total {
        let start = seq * CHUNK_LEN;
        let end = (start + CHUNK_LEN).min(data.len());
        let chunk = &data[start..end];
        let mut p = [0u8; PACKET_LEN];
        p[0] = cmd;
        p[1] = total as u8;
        p[2] = seq as u8;
        let mut yy = match cmd {
            1 => 14 + (layer << 4),
            _ => 14,
        };
        if seq == total - 1 {
            if cmd == 4 {
                yy = chunk.len() as u8;
            } else if cmd == 1 || cmd == 9 {
                let mut last = chunk.len() as i32 - 1;
                while last >= 0 && chunk[last as usize] == 0 {
                    last -= 1;
                }
                yy = if last < 0 {
                    0
                } else {
                    14 + if cmd == 1 { layer << 4 } else { 0 }
                        - (chunk.len() as i32 - 1 - last) as u8
                };
            }
        }
        p[3] = yy;
        p[4..4 + chunk.len()].copy_from_slice(chunk);
        p[PACKET_LEN - 1] = checksum(&p);
        out.push(p);
    }
    Ok(out)
}

pub fn parse_performance(perf: &[u8], light: &[u8]) -> KeyboardPerformance {
    let mode = perf.get(10).copied().unwrap_or(0);
    let slot = brightness_offset(mode);
    let brightness = slot.and_then(|i| perf.get(i)).copied().unwrap_or(0);
    let packed = slot.and_then(|i| perf.get(i + 1)).copied().unwrap_or(0);
    let (hi, lo) = (packed >> 4, packed & 0x0F);
    let speed = if hi <= 4 { hi } else { lo };
    let multi = if hi > 4 { hi } else { lo };
    let color = color_offset(mode)
        .and_then(|i| light.get(i..i + 3))
        .map(|s| [s[0], s[1], s[2]])
        .unwrap_or([0, 0, 0]);
    let rate = match perf.get(1).copied().unwrap_or(0) {
        1 => 250,
        2 => 500,
        3 => 1000,
        _ => 0,
    };
    let latency = match perf.get(3).copied().unwrap_or(2) {
        0 => "speed",
        3 => "special",
        _ => "standard",
    };
    KeyboardPerformance {
        rate_hz: rate,
        sleep_minutes: perf.get(24).copied().unwrap_or(0) / 2,
        latency: latency.to_string(),
        win_key_locked: perf.get(15).copied().unwrap_or(0) != 0,
        mac_mode: perf.get(27).copied().unwrap_or(0) != 0,
        light_type: perf.get(9).copied().unwrap_or(0),
        light_mode: mode,
        brightness,
        speed,
        multi_color: multi != 0,
        color,
    }
}

pub struct KeyboardDevice {
    link: Link,
    pub info: DeviceInfo,
    perf: Vec<u8>,
    light: Vec<u8>,
    layers: [Vec<u8>; 3],
    battery: Battery,
}

impl KeyboardDevice {
    pub fn open(key: &str) -> Result<Self, DeviceError> {
        let api = HidApi::new()?;
        let target = api.device_list().find(|d| {
            d.vendor_id() == VID
                && d.product_id() == PID
                && d.usage_page() == 0xFF02
                && summary_key(d.serial_number()) == key
        });
        let info = target.ok_or_else(|| {
            DeviceError::not_found("2.4G 键盘接收器不在线：请插好接收器并把键盘切到 2.4G")
        })?;
        let dev = info.open_device(&api)?;
        let link = Link {
            dev,
            last_write: None,
        };

        let signature = link.read_data(5, 0, 6)?;
        if signature.len() < 6 || signature[..6] != K99_V2_SIGNATURE {
            return Err(DeviceError::unsupported(format!(
                "接收器返回的键盘型号标识是 {}，目前只支持 K99 V2",
                signature
                    .iter()
                    .take(6)
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )));
        }

        let battery_data = link.read_data(0x4A, 0, 2)?;
        let battery = Battery {
            percent: battery_data[0].clamp(1, 100),
            charging: battery_data[1] & 0x0F != 0,
        };
        let mut perf = link.read_data(0x44, 0, PERF_LEN)?;
        perf.truncate(PERF_LEN);
        let light = link.read_data(0x49, 0, LIGHT_LEN)?;
        let mut layers: [Vec<u8>; 3] = [
            vec![0; KEY_LAYER_LEN],
            vec![0; KEY_LAYER_LEN],
            vec![0; KEY_LAYER_LEN],
        ];
        for (layer, slot) in layers.iter_mut().enumerate() {
            *slot = link.read_data(0x41, layer as u8, KEY_LAYER_LEN)?;
            slot.truncate(KEY_LAYER_LEN);
        }

        let info = DeviceInfo {
            key: key.to_string(),
            kind: DeviceKind::Keyboard,
            model: Some("K99 V2".to_string()),
            product_name: info.product_string().unwrap_or("MCHOSE K99 V2").to_string(),
            vendor_id: VID,
            product_id: PID,
            serial: info.serial_number().map(str::to_string),
            connection: ConnectionKind::Wireless,
            supported: true,
            capabilities: keyboard_capabilities(),
            firmware: None,
            build_date: None,
            is_demo: false,
            note: None,
        };

        Ok(Self {
            link,
            info,
            perf,
            light,
            layers,
            battery,
        })
    }

    pub fn read_live(&mut self) -> Result<LiveStatus, DeviceError> {
        let data = self.link.read_data(0x4A, 0, 2)?;
        self.battery = Battery {
            percent: data[0].clamp(1, 100),
            charging: data[1] & 0x0F != 0,
        };
        Ok(LiveStatus {
            online: true,
            battery: Some(self.battery),
        })
    }

    pub fn refresh(&mut self) -> Result<KeyboardSnapshot, DeviceError> {
        self.battery = {
            let data = self.link.read_data(0x4A, 0, 2)?;
            Battery {
                percent: data[0].clamp(1, 100),
                charging: data[1] & 0x0F != 0,
            }
        };
        self.perf = self.link.read_data(0x44, 0, PERF_LEN)?;
        self.perf.truncate(PERF_LEN);
        self.light = self.link.read_data(0x49, 0, LIGHT_LEN)?;
        for (layer, slot) in self.layers.iter_mut().enumerate() {
            *slot = self.link.read_data(0x41, layer as u8, KEY_LAYER_LEN)?;
            slot.truncate(KEY_LAYER_LEN);
        }
        Ok(self.snapshot())
    }

    pub fn snapshot(&self) -> KeyboardSnapshot {
        KeyboardSnapshot {
            info: self.info.clone(),
            online: true,
            battery: Some(self.battery),
            performance: self.performance(),
            layers: self.layers_view(),
            light_modes: light_modes(),
            read_at: unix_now(),
        }
    }

    fn performance(&self) -> KeyboardPerformance {
        parse_performance(&self.perf, &self.light)
    }

    fn layers_view(&self) -> Vec<Vec<[u8; 4]>> {
        self.layers
            .iter()
            .map(|layer| {
                (0..KEY_COUNT)
                    .map(|i| {
                        let o = i * 4;
                        [
                            layer.get(o).copied().unwrap_or(0),
                            layer.get(o + 1).copied().unwrap_or(0),
                            layer.get(o + 2).copied().unwrap_or(0),
                            layer.get(o + 3).copied().unwrap_or(0),
                        ]
                    })
                    .collect()
            })
            .collect()
    }

    pub fn apply(&mut self, input: &KeyboardInput) -> Result<KeyboardSnapshot, DeviceError> {
        validate(input)?;

        let mut perf = self.perf.clone();
        let mode = input.performance.light_mode;
        perf[1] = match input.performance.rate_hz {
            250 => 1,
            500 => 2,
            1000 => 3,
            other => return Err(DeviceError::invalid(format!("K99 V2 不支持 {other} Hz"))),
        };
        perf[3] = match input.performance.latency.as_str() {
            "speed" => 0,
            "special" => 3,
            _ => 2,
        };
        perf[15] = u8::from(input.performance.win_key_locked);
        perf[24] = input.performance.sleep_minutes.saturating_mul(2);
        perf[27] = u8::from(input.performance.mac_mode);
        perf[9] = input.performance.light_type;
        perf[10] = mode;
        if let Some(offset) = brightness_offset(mode) {
            if offset < PERF_LEN {
                perf[offset] = input.performance.brightness.min(100);
            }
            if offset + 1 < PERF_LEN {
                perf[offset + 1] =
                    (input.performance.speed.min(4) << 4) | u8::from(input.performance.multi_color);
            }
        }
        if perf != self.perf {
            self.link.write_data(4, &perf, 0)?;
            std::thread::sleep(Duration::from_millis(350));
            self.perf = self.link.read_data(0x44, 0, PERF_LEN)?;
            self.perf.truncate(PERF_LEN);
        }

        let mut light = self.light.clone();
        if let Some(offset) = color_offset(mode) {
            if offset + 2 < LIGHT_LEN {
                light[offset..offset + 3].copy_from_slice(&input.performance.color);
            }
        }
        if light != self.light {
            let mut payload = light.clone();
            payload.extend_from_slice(&[0u8; 16]);
            payload.extend_from_slice(&[0x5A, 0xA5, 0, 0, 0, 0]);
            self.link.write_data(9, &payload, 0)?;
            std::thread::sleep(Duration::from_millis(350));
            self.light = self.link.read_data(0x49, 0, LIGHT_LEN)?;
        }

        for (index, wanted) in input.layers.iter().enumerate() {
            let mut data = Vec::with_capacity(KEY_LAYER_LEN + 60);
            for code in wanted {
                data.extend_from_slice(code);
            }
            data.extend_from_slice(&[0u8; 58]);
            data.extend_from_slice(&[0x5A, 0xA5]);
            if data[..KEY_LAYER_LEN] != self.layers[index][..] {
                self.link.write_data(1, &data, index as u8)?;
                std::thread::sleep(Duration::from_millis(350));
                self.layers[index] = self.link.read_data(0x41, index as u8, KEY_LAYER_LEN)?;
                self.layers[index].truncate(KEY_LAYER_LEN);
            }
        }

        Ok(self.snapshot())
    }
}

pub fn validate(input: &KeyboardInput) -> Result<(), DeviceError> {
    if !matches!(input.performance.rate_hz, 250 | 500 | 1000) {
        return Err(DeviceError::invalid("K99 V2 只支持 250/500/1000 Hz"));
    }
    if input.performance.sleep_minutes > 30 {
        return Err(DeviceError::invalid("K99 V2 休眠时间最多 30 分钟"));
    }
    if !["speed", "standard", "special"].contains(&input.performance.latency.as_str()) {
        return Err(DeviceError::invalid("未知的延迟模式"));
    }
    if input.performance.brightness > 100 || input.performance.speed > 4 {
        return Err(DeviceError::invalid("亮度或灯效速度超出范围"));
    }
    if input.layers.len() != 3 {
        return Err(DeviceError::invalid("键盘必须提供 3 层按键"));
    }
    for layer in &input.layers {
        if layer.len() != KEY_COUNT {
            return Err(DeviceError::invalid(format!(
                "每层按键必须是 {KEY_COUNT} 个"
            )));
        }
    }
    Ok(())
}

pub fn keyboard_capabilities() -> Capabilities {
    let mut caps = Capabilities::unsupported();
    caps.battery = true;
    caps.max_stages = 0;
    caps
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k99_signature_matches() {
        assert_eq!(K99_V2_SIGNATURE, [0x03, 0x00, 0x00, 0x00, 0x01, 0xDF]);
    }

    #[test]
    fn read_request_has_valid_checksum() {
        let p = read_request(5, 0);
        assert_eq!(p[0], 5);
        assert_eq!(p[1], 1);
        assert_eq!(p[18], checksum(&p));
    }

    #[test]
    fn write_packets_cover_the_block() {
        let data = vec![0xAB; 128];
        let packets = write_packets(4, &data, 0).unwrap();
        assert_eq!(packets.len(), 10);
        assert_eq!(packets[0][0], 4);
        assert_eq!(packets[0][1], 10);
        assert_eq!(packets[9][3], 2);
        for p in &packets {
            assert_eq!(p[18], checksum(p));
        }
    }

    #[test]
    fn layer_write_has_trailing_marker() {
        let mut data = vec![0u8; KEY_LAYER_LEN];
        data[0] = 0x04;
        data.extend_from_slice(&[0u8; 58]);
        data.extend_from_slice(&[0x5A, 0xA5]);
        let packets = write_packets(1, &data, 0).unwrap();
        assert_eq!(data.len(), 512);
        assert_eq!(packets.len(), 37);
        assert_eq!(packets.last().unwrap()[0], 1);
    }

    #[test]
    fn performance_offsets_match_verified_k99_block() {
        let mut perf = vec![0u8; PERF_LEN];
        perf[1] = 3;
        perf[3] = 2;
        perf[10] = 11;
        perf[15] = 1;
        perf[24] = 6;
        perf[27] = 1;
        let off = brightness_offset(11).unwrap();
        perf[off] = 80;
        perf[off + 1] = 0x27;
        let light = vec![0u8; LIGHT_LEN];
        let p = parse_performance(&perf, &light);
        assert_eq!(p.rate_hz, 1000);
        assert_eq!(p.sleep_minutes, 3);
        assert!(p.win_key_locked);
        assert!(p.mac_mode);
        assert_eq!(p.light_mode, 11);
        assert_eq!(p.brightness, 80);
        assert_eq!(p.speed, 2);
        assert!(p.multi_color);
    }
}
