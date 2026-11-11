//! MCHOSE A7 V3 Ultra+ (and compatible 0x3837 mice).
//!
//! This family uses report `0x4D` over HID output/input (not the 0x55 frame
//! used by the G3 V2). The frame layout was verified against a real
//! `MCHOSE A7 V3 Ultra+` on macOS:
//!
//! ```text
//! 4D 01 flag len cmd_lo cmd_hi 00 00 payload... xor
//! ```
//!
//! `flag = 1` means the last byte is XOR(frame[2..8+len]). Read commands have
//! bit 15 clear (`0x0900`, `0x0002`, ...), writes set it (`0x0102`, `0x0103`).

use crate::error::DeviceError;
use crate::model::{
    Battery, ButtonAction, Capabilities, ConnectionKind, DeviceInfo, DeviceKind, DeviceSummary,
    DpiRange, LiveStatus, Lod, MouseConfig, Snapshot,
};
use hidapi::{HidApi, HidDevice};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const REPORT_ID: u8 = 0x4D;
pub const PIDS: [u16; 3] = [0x4033, 0x1014, 0x1018];
pub const POLLING_RATES: [u32; 6] = [125, 500, 1000, 2000, 4000, 8000];
const MIN_GAP: Duration = Duration::from_millis(28);

pub fn is_a7_pid(pid: u16) -> bool {
    PIDS.contains(&pid)
}

pub fn is_a7_name(name: &str) -> bool {
    name.to_ascii_uppercase().contains("A7 V3 ULTRA+")
}

pub fn summary_key(vid: u16, pid: u16, serial: Option<&str>) -> String {
    format!("a7:{vid:04x}:{pid:04x}:{}", serial.unwrap_or(""))
}

pub fn capabilities() -> Capabilities {
    Capabilities {
        dpi: true,
        polling: true,
        debounce: true,
        sleep: true,
        // 这个型号的 LOD 是 0.7–1.7 mm 的连续档位；当前的 low/high 模型只覆盖 G3，
        // 先不开放，避免写出错误值。
        lod: false,
        buttons: false,
        scroll_direction: false,
        battery: true,
        firmware: false,
        dpi_range: DpiRange {
            min: 200,
            max: 50_000,
            step: 50,
        },
        polling_rates: POLLING_RATES.to_vec(),
        max_stages: 6,
    }
}

pub fn scan() -> Result<Vec<DeviceSummary>, DeviceError> {
    let api = HidApi::new()?;
    let mut out: Vec<DeviceSummary> = Vec::new();
    for d in api.device_list() {
        if d.vendor_id() != crate::MCHOSE_VENDOR_ID || !is_a7_pid(d.product_id()) {
            continue;
        }
        if d.usage_page() != 0xFF01 {
            continue;
        }
        let name = d.product_string().unwrap_or("MCHOSE A7 V3 Ultra+");
        if !is_a7_name(name) {
            continue;
        }
        let key = summary_key(d.vendor_id(), d.product_id(), d.serial_number());
        if out.iter().any(|s| s.key == key) {
            continue;
        }
        out.push(DeviceSummary {
            key,
            kind: DeviceKind::Mouse,
            name: "MCHOSE A7 V3 Ultra+".to_string(),
            model: Some("A7 V3 Ultra+".to_string()),
            serial: d.serial_number().map(str::to_string),
            vendor_id: d.vendor_id(),
            product_id: d.product_id(),
            connection: ConnectionKind::Wired,
            supported: true,
            status_note: None,
            is_demo: false,
        });
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A7Status {
    pub battery: u8,
    pub charging: bool,
    pub connection: u8,
    pub profile_count: u8,
}

pub struct A7Device {
    dev: HidDevice,
    pub info: DeviceInfo,
    settings: [u8; 19],
    dpi: [u8; 17],
    status: A7Status,
    last_cmd: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A7Frame {
    pub command: u16,
    pub payload: Vec<u8>,
}

pub fn encode_frame(command: u16, payload: &[u8]) -> Result<[u8; 64], DeviceError> {
    if payload.len() + 9 > 64 {
        return Err(DeviceError::invalid("A7 帧负载超过 55 字节"));
    }
    let mut f = [0u8; 64];
    f[0] = REPORT_ID;
    f[1] = 1;
    f[2] = 1;
    f[3] = payload.len() as u8;
    f[4] = command as u8;
    f[5] = (command >> 8) as u8;
    f[8..8 + payload.len()].copy_from_slice(payload);
    f[8 + payload.len()] = f[2..8 + payload.len()].iter().fold(0u8, |a, b| a ^ b);
    Ok(f)
}

pub fn decode_frame(raw: &[u8]) -> Result<A7Frame, DeviceError> {
    if raw.len() < 8 || raw[0] != REPORT_ID || raw[1] != 1 {
        return Err(DeviceError::protocol("不是 A7 的 0x4D 帧"));
    }
    let len = raw[3] as usize;
    if raw.len() < 8 + len {
        return Err(DeviceError::protocol("A7 回应被截断"));
    }
    let command = u16::from_le_bytes([raw[4], raw[5]]);
    let payload = raw[8..8 + len].to_vec();
    if raw[2] & 1 == 1 && raw.len() > 8 + len {
        let calculated = raw[2..8 + len].iter().fold(0u8, |a, b| a ^ b);
        let received = raw[8 + len];
        // A7 V3 firmware <= 6 returns 0901 without the advertised XOR.
        let version_lenient =
            command == 0x0901 && len == 4 && raw[8 + len..].iter().all(|b| *b == 0);
        if calculated != received && !version_lenient {
            return Err(DeviceError::protocol(format!(
                "A7 回应校验失败：期望 {calculated:02X}，收到 {received:02X}"
            )));
        }
    }
    Ok(A7Frame { command, payload })
}

fn rate_decode(raw: u8) -> u8 {
    if raw > 1 {
        raw - 1
    } else {
        raw
    }
}

fn rate_encode(index: u8) -> u8 {
    if index > 0 {
        index + 1
    } else {
        0
    }
}

fn polling_from_index(index: u8) -> u32 {
    POLLING_RATES.get(index as usize).copied().unwrap_or(1000)
}

fn u16_at(data: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([data[at], data[at + 1]])
}

impl A7Device {
    pub fn open(key: &str) -> Result<Self, DeviceError> {
        let api = HidApi::new()?;
        let target = api.device_list().find(|d| {
            d.vendor_id() == crate::MCHOSE_VENDOR_ID
                && is_a7_pid(d.product_id())
                && d.usage_page() == 0xFF01
                && summary_key(d.vendor_id(), d.product_id(), d.serial_number()) == key
        });
        let info = target
            .ok_or_else(|| DeviceError::not_found("A7 V3 Ultra+ 不在线：请重新插拔 USB 数据线"))?;
        let dev = info.open_device(&api)?;
        let product_name = info
            .product_string()
            .unwrap_or("MCHOSE A7 V3 Ultra+")
            .to_string();
        let device = Self {
            dev,
            info: DeviceInfo {
                key: key.to_string(),
                kind: DeviceKind::Mouse,
                model: Some("A7 V3 Ultra+".to_string()),
                product_name,
                vendor_id: info.vendor_id(),
                product_id: info.product_id(),
                serial: info.serial_number().map(str::to_string),
                connection: ConnectionKind::Wired,
                supported: true,
                capabilities: capabilities(),
                firmware: None,
                build_date: None,
                is_demo: false,
                note: Some("A7 系列使用 0x4D 帧协议；按键改键与灯光暂未开放。".to_string()),
            },
            settings: [0; 19],
            dpi: [0; 17],
            status: A7Status {
                battery: 0,
                charging: false,
                connection: 0,
                profile_count: 0,
            },
            last_cmd: Instant::now(),
        };
        let mut device = device;
        device.read_state()?;
        Ok(device)
    }

    fn pace(&mut self) {
        let elapsed = self.last_cmd.elapsed();
        if elapsed < MIN_GAP {
            std::thread::sleep(MIN_GAP - elapsed);
        }
        self.last_cmd = Instant::now();
    }

    fn read(&mut self, command: u16, payload: &[u8]) -> Result<Vec<u8>, DeviceError> {
        let mut last = DeviceError::timeout(format!("A7 命令 0x{command:04X} 超时"));
        for _ in 0..3 {
            self.pace();
            let frame = encode_frame(command, payload)?;
            self.dev
                .write(&frame)
                .map_err(|e| DeviceError::hid(format!("A7 写入失败：{e}")))?;
            let deadline = Instant::now() + Duration::from_millis(1500);
            while Instant::now() < deadline {
                let remaining = (deadline - Instant::now()).as_millis().max(1) as i32;
                let mut buf = [0u8; 256];
                match self.dev.read_timeout(&mut buf, remaining) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(frame) = decode_frame(&buf[..n]) {
                            if frame.command == command {
                                return Ok(frame.payload);
                            }
                            // 0x0600 是设备事件，继续等目标命令
                        }
                    }
                    Err(e) => {
                        last = DeviceError::hid(format!("A7 读取失败：{e}"));
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(120));
        }
        Err(last)
    }

    fn write(&mut self, command: u16, payload: &[u8]) -> Result<(), DeviceError> {
        let response = self.read(command, payload)?;
        // 写命令的回应可能是空负载；只要帧命令号匹配就视为确认。
        let _ = response;
        Ok(())
    }

    fn read_status(&mut self) -> Result<A7Status, DeviceError> {
        let data = self.read(0x0900, &[])?;
        if data.len() < 14 {
            return Err(DeviceError::protocol("A7 状态回应短于 14 字节"));
        }
        Ok(A7Status {
            battery: data[12],
            charging: data[11] != 0,
            connection: data[10],
            profile_count: data[4],
        })
    }

    fn read_settings(&mut self) -> Result<[u8; 19], DeviceError> {
        let data = self.read(0x0002, &[])?;
        let mut out = [0u8; 19];
        let n = data.len().min(19);
        out[..n].copy_from_slice(&data[..n]);
        Ok(out)
    }

    fn read_dpi(&mut self, profile: u8, direction: u8) -> Result<[u8; 17], DeviceError> {
        let data = self.read(0x0003, &[profile, direction])?;
        let mut out = [0u8; 17];
        let n = data.len().min(17);
        out[..n].copy_from_slice(&data[..n]);
        Ok(out)
    }

    fn read_state(&mut self) -> Result<(), DeviceError> {
        self.status = self.read_status()?;
        self.settings = self.read_settings()?;
        let profile = self.settings[0].min(self.status.profile_count.saturating_sub(1));
        self.dpi = self.read_dpi(profile, 0)?;
        Ok(())
    }

    pub fn read_live(&mut self) -> Result<LiveStatus, DeviceError> {
        let status = self.read_status()?;
        self.status = status.clone();
        Ok(LiveStatus {
            online: true,
            battery: Some(Battery {
                percent: status.battery,
                charging: status.charging,
            }),
        })
    }

    fn config(&self) -> Result<MouseConfig, DeviceError> {
        let count = self.dpi[2].clamp(1, 6);
        let active = (self.dpi[3] + 1).clamp(1, count);
        let mut stages = [400, 800, 1600, 3200, 6400, 50_000];
        for (i, stage) in stages.iter_mut().enumerate() {
            let v = u16_at(&self.dpi, 5 + i * 2);
            if v > 0 {
                *stage = v;
            }
        }
        let rate_index = rate_decode(self.settings[1] >> 4);
        Ok(MouseConfig {
            polling_hz: polling_from_index(rate_index),
            stage_count: count,
            active_stage: active,
            stages,
            lod: Lod::High,
            debounce_ms: self.settings[7].min(20),
            sleep_minutes: self.settings[3],
            scroll_reversed: false,
            sensor_flags: self.settings[5],
            highspeed_mode: 0,
            sensor_angle: self.settings[6] as i8,
        })
    }

    pub fn snapshot(&mut self) -> Result<Snapshot, DeviceError> {
        self.read_state()?;
        let config = self.config()?;
        Ok(Snapshot {
            info: self.info.clone(),
            online: true,
            battery: Some(Battery {
                percent: self.status.battery,
                charging: self.status.charging,
            }),
            config: Some(config),
            buttons: None,
            factory_config: factory_config(),
            factory_buttons: Vec::new(),
            read_at: unix_now(),
        })
    }

    pub fn apply(
        &mut self,
        config: &MouseConfig,
        _buttons: &[ButtonAction],
    ) -> Result<Snapshot, DeviceError> {
        validate_a7(config)?;

        let rate_index = POLLING_RATES
            .iter()
            .position(|v| *v == config.polling_hz)
            .ok_or_else(|| DeviceError::invalid("A7 不支持这个回报率"))?
            as u8;
        let old_rate = rate_decode(self.settings[1] >> 4);
        let settings_changed = old_rate != rate_index
            || self.settings[3] != config.sleep_minutes
            || self.settings[7] != config.debounce_ms;

        if settings_changed {
            let mut next = self.settings;
            let dpi_index = self.dpi[3].min(5);
            next[1] = (rate_encode(rate_index) << 4) | (dpi_index & 0x0F);
            next[2] = (rate_encode(rate_index) << 4) | (dpi_index & 0x0F);
            next[3] = config.sleep_minutes;
            next[7] = config.debounce_ms;
            next[8] = config.debounce_ms;
            self.write(0x0102, &next)?;
            self.settings = next;
            std::thread::sleep(Duration::from_millis(60));
        }

        let stages_changed = self.dpi[2] != config.stage_count
            || self.dpi[3] + 1 != config.active_stage
            || (0..6).any(|i| u16_at(&self.dpi, 5 + i * 2) != config.stages[i]);
        if stages_changed {
            let payload = encode_dpi_payload(self.dpi[0].min(2), config)?;
            self.write(0x0103, &payload)?;
            std::thread::sleep(Duration::from_millis(60));
            let profile = self.dpi[0].min(2);
            self.dpi = self.read_dpi(profile, 0)?;
        }

        self.snapshot()
    }
}

/// A7 DPI 写包的字段顺序与读回格式不同：
///   读: [profile, ?, count, selected, separate_y, x...]
///   写: [profile, direction, separate_y, count, selected, x...]
pub fn encode_dpi_payload(profile: u8, config: &MouseConfig) -> Result<[u8; 17], DeviceError> {
    let mut payload = [0u8; 17];
    payload[0] = profile.min(2);
    payload[1] = 0;
    payload[2] = 0; // separate Y 关闭；当前界面只编辑 X 轴
    payload[3] = config.stage_count;
    payload[4] = config.active_stage.saturating_sub(1);
    for (i, value) in config.stages.iter().enumerate() {
        payload[5 + i * 2..7 + i * 2].copy_from_slice(&value.to_le_bytes());
    }
    Ok(payload)
}

pub fn validate_a7(config: &MouseConfig) -> Result<(), DeviceError> {
    if !(1..=6).contains(&config.stage_count) {
        return Err(DeviceError::invalid("A7 的 DPI 档位数量必须在 1–6 之间"));
    }
    if config.active_stage < 1 || config.active_stage > config.stage_count {
        return Err(DeviceError::invalid("A7 的当前档位超出已启用范围"));
    }
    for (i, dpi) in config
        .stages
        .iter()
        .enumerate()
        .take(config.stage_count as usize)
    {
        if *dpi < 200 || *dpi > 50_000 || *dpi % 50 != 0 {
            return Err(DeviceError::invalid(format!(
                "A7 第 {} 档 DPI 必须在 200–50000 之间、以 50 为步进",
                i + 1
            )));
        }
    }
    if !POLLING_RATES.contains(&config.polling_hz) {
        return Err(DeviceError::invalid("A7 不支持这个回报率"));
    }
    if config.debounce_ms > 20 {
        return Err(DeviceError::invalid("A7 防抖必须在 0–20 ms"));
    }
    if config.sleep_minutes > 100 {
        return Err(DeviceError::invalid("A7 休眠必须在 0–100 分钟"));
    }
    Ok(())
}

pub fn factory_config() -> MouseConfig {
    MouseConfig {
        polling_hz: 1000,
        stage_count: 6,
        active_stage: 3,
        stages: [400, 800, 1600, 3200, 6400, 50_000],
        lod: Lod::High,
        debounce_ms: 8,
        sleep_minutes: 3,
        scroll_reversed: false,
        sensor_flags: 0,
        highspeed_mode: 0,
        sensor_angle: 0,
    }
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

    fn sample_frame() -> Vec<u8> {
        // 实机 0x0900 状态回包
        let mut f = vec![0u8; 64];
        f[0] = 0x4D;
        f[1] = 1;
        f[2] = 1;
        f[3] = 16;
        f[4] = 0x00;
        f[5] = 0x09;
        f[8..24].copy_from_slice(&[
            0x37, 0x38, 0x26, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x10, 0x02, 0x00, 0x4d, 0x00,
            0x08, 0xe4,
        ]);
        f[24] = f[2..24].iter().fold(0u8, |a, b| a ^ b);
        f
    }

    #[test]
    fn decodes_real_status_frame() {
        let frame = sample_frame();
        let decoded = decode_frame(&frame).unwrap();
        assert_eq!(decoded.command, 0x0900);
        let status = &decoded.payload;
        assert_eq!(u16_at(status, 0), 0x3837);
        assert_eq!(status[12], 0x4d);
    }

    #[test]
    fn encodes_frame_with_checksum() {
        let frame = encode_frame(0x0900, &[]).unwrap();
        assert_eq!(&frame[..8], &[0x4D, 1, 1, 0, 0x00, 0x09, 0, 0]);
        assert_eq!(frame[8], 0x08);
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut frame = sample_frame();
        frame[24] ^= 0xFF;
        assert!(decode_frame(&frame).is_err());
    }

    #[test]
    fn dpi_write_payload_uses_write_field_order() {
        let mut cfg = factory_config();
        cfg.stage_count = 3;
        cfg.active_stage = 2;
        cfg.stages = [400, 800, 1600, 3200, 6400, 50000];
        let p = encode_dpi_payload(0, &cfg).unwrap();
        assert_eq!(&p[..5], &[0, 0, 0, 3, 1]);
        assert_eq!(u16::from_le_bytes([p[5], p[6]]), 400);
        assert_eq!(u16::from_le_bytes([p[7], p[8]]), 800);
        assert_eq!(u16::from_le_bytes([p[9], p[10]]), 1600);
    }

    #[test]
    fn validates_a7_ranges() {
        validate_a7(&factory_config()).unwrap();
        let mut bad = factory_config();
        bad.stages[0] = 15_000;
        bad.polling_hz = 8000;
        validate_a7(&bad).unwrap();
        bad.stages[0] = 150;
        assert!(validate_a7(&bad).is_err());
        bad.stages[0] = 1600;
        bad.polling_hz = 100;
        assert!(validate_a7(&bad).is_err());
    }
}
