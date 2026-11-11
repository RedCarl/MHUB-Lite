use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const MIN_DPI: u16 = 100;
pub const MAX_DPI: u16 = 12_000;
pub const DPI_STEP: u16 = 100;

/// 协议里的回报率档位索引（0 = 125Hz … 6 = 8000Hz）。
pub const POLLING_TABLE: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKind {
    Mouse,
    Keyboard,
}

impl DeviceKind {
    pub fn label_zh(self) -> &'static str {
        match self {
            DeviceKind::Mouse => "鼠标",
            DeviceKind::Keyboard => "键盘",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionKind {
    Wired,
    Wireless,
    /// 未知型号：只识别到 HID 设备，无法确认它是有线还是接收器。
    Unknown,
}

impl ConnectionKind {
    pub fn label_zh(self) -> &'static str {
        match self {
            ConnectionKind::Wired => "USB 有线",
            ConnectionKind::Wireless => "2.4G 无线",
            ConnectionKind::Unknown => "连接方式未知",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lod {
    /// 1 mm
    Low,
    /// 2 mm
    High,
}

impl Lod {
    pub fn to_u8(self) -> u8 {
        match self {
            Lod::Low => 0,
            Lod::High => 1,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        if v == 0 {
            Lod::Low
        } else {
            Lod::High
        }
    }

    pub fn label_zh(self) -> &'static str {
        match self {
            Lod::Low => "1 mm",
            Lod::High => "2 mm",
        }
    }
}

/// 鼠标按键动作。
///
/// JSON 里是一个短字符串（`left` / `forward` / …），未识别的硬件码保留成
/// `custom:20030000`，读回写时不会被吞掉。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    Left,
    Right,
    Middle,
    Forward,
    Backward,
    Disabled,
    Custom([u8; 4]),
}

pub const DEFAULT_BUTTONS: [ButtonAction; 5] = [
    ButtonAction::Left,
    ButtonAction::Right,
    ButtonAction::Middle,
    ButtonAction::Forward,
    ButtonAction::Backward,
];

impl ButtonAction {
    pub fn to_wire(self) -> [u8; 4] {
        match self {
            ButtonAction::Left => [0x20, 0x01, 0x00, 0x00],
            ButtonAction::Right => [0x20, 0x02, 0x00, 0x00],
            ButtonAction::Middle => [0x20, 0x04, 0x00, 0x00],
            ButtonAction::Backward => [0x20, 0x08, 0x00, 0x00],
            ButtonAction::Forward => [0x20, 0x10, 0x00, 0x00],
            ButtonAction::Disabled => [0x20, 0x00, 0x00, 0x00],
            ButtonAction::Custom(raw) => raw,
        }
    }

    pub fn from_wire(bytes: [u8; 4]) -> Self {
        match bytes {
            [0x20, 0x01, 0x00, 0x00] => ButtonAction::Left,
            [0x20, 0x02, 0x00, 0x00] => ButtonAction::Right,
            [0x20, 0x04, 0x00, 0x00] => ButtonAction::Middle,
            [0x20, 0x08, 0x00, 0x00] => ButtonAction::Backward,
            [0x20, 0x10, 0x00, 0x00] => ButtonAction::Forward,
            [0x20, 0x00, 0x00, 0x00] => ButtonAction::Disabled,
            other => ButtonAction::Custom(other),
        }
    }

    pub fn code(self) -> String {
        match self {
            ButtonAction::Left => "left".into(),
            ButtonAction::Right => "right".into(),
            ButtonAction::Middle => "middle".into(),
            ButtonAction::Forward => "forward".into(),
            ButtonAction::Backward => "backward".into(),
            ButtonAction::Disabled => "disabled".into(),
            ButtonAction::Custom(raw) => format!(
                "custom:{:02x}{:02x}{:02x}{:02x}",
                raw[0], raw[1], raw[2], raw[3]
            ),
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "left" | "lmb" => Some(ButtonAction::Left),
            "right" | "rmb" => Some(ButtonAction::Right),
            "middle" | "mmb" => Some(ButtonAction::Middle),
            "forward" | "mouse5" => Some(ButtonAction::Forward),
            "backward" | "back" | "mouse4" => Some(ButtonAction::Backward),
            "disabled" | "none" | "off" => Some(ButtonAction::Disabled),
            other => other.strip_prefix("custom:").and_then(|hex| {
                if hex.len() != 8 {
                    return None;
                }
                let mut raw = [0u8; 4];
                for (i, byte) in raw.iter_mut().enumerate() {
                    *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
                }
                Some(ButtonAction::Custom(raw))
            }),
        }
    }

    pub fn label_zh(self) -> &'static str {
        match self {
            ButtonAction::Left => "左键",
            ButtonAction::Right => "右键",
            ButtonAction::Middle => "中键",
            ButtonAction::Forward => "前进键",
            ButtonAction::Backward => "后退键",
            ButtonAction::Disabled => "禁用",
            ButtonAction::Custom(_) => "自定义动作",
        }
    }
}

impl fmt::Display for ButtonAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code())
    }
}

impl Serialize for ButtonAction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.code())
    }
}

impl<'de> Deserialize<'de> for ButtonAction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        ButtonAction::from_code(&s).ok_or_else(|| de::Error::custom(format!("未知按键动作：{s}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpiRange {
    pub min: u16,
    pub max: u16,
    pub step: u16,
}

impl Default for DpiRange {
    fn default() -> Self {
        Self {
            min: MIN_DPI,
            max: MAX_DPI,
            step: DPI_STEP,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub dpi: bool,
    pub polling: bool,
    pub debounce: bool,
    pub sleep: bool,
    pub lod: bool,
    pub buttons: bool,
    pub scroll_direction: bool,
    pub battery: bool,
    pub firmware: bool,
    pub dpi_range: DpiRange,
    pub polling_rates: Vec<u32>,
    pub max_stages: u8,
}

impl Capabilities {
    /// 未验证型号：只读、不提供任何写入口。
    pub fn unsupported() -> Self {
        Self {
            dpi: false,
            polling: false,
            debounce: false,
            sleep: false,
            lod: false,
            buttons: false,
            scroll_direction: false,
            battery: false,
            firmware: false,
            dpi_range: DpiRange::default(),
            polling_rates: Vec::new(),
            max_stages: 0,
        }
    }
}

/// 设备上实际保存的一份配置。
///
/// `sensor_flags` / `highspeed_mode` / `sensor_angle` 当前没有在界面里开放，
/// 但必须原样读出来、原样写回去 —— 否则一次「保存」就会把它们清零。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MouseConfig {
    pub polling_hz: u32,
    pub stage_count: u8,
    /// 1 基：第 1 档 … 第 6 档
    pub active_stage: u8,
    /// 固定 6 个槽位；只有前 `stage_count` 个在界面上出现
    pub stages: [u16; 6],
    pub lod: Lod,
    pub debounce_ms: u8,
    /// 0 表示永不休眠
    pub sleep_minutes: u8,
    pub scroll_reversed: bool,
    /// 以下为「读出来但界面不碰」的字段
    pub sensor_flags: u8,
    pub highspeed_mode: u8,
    pub sensor_angle: i8,
}

impl Default for MouseConfig {
    /// G3 V2 的原厂默认值（与官方网页驱动逐项核对过）。
    fn default() -> Self {
        Self {
            polling_hz: 250,
            stage_count: 6,
            active_stage: 3,
            stages: [400, 800, 1600, 3200, 6400, 12_000],
            lod: Lod::High,
            debounce_ms: 8,
            sleep_minutes: 3,
            scroll_reversed: false,
            sensor_flags: 0,
            highspeed_mode: 0,
            sensor_angle: 0,
        }
    }
}

impl MouseConfig {
    pub fn validate(&self) -> Result<(), crate::DeviceError> {
        use crate::error::DeviceError;

        if !(1..=6).contains(&self.stage_count) {
            return Err(DeviceError::invalid("启用的 DPI 档位数量必须在 1–6 之间"));
        }
        if self.active_stage < 1 || self.active_stage > self.stage_count {
            return Err(DeviceError::invalid("当前档位必须落在已启用的档位范围内"));
        }
        for (i, dpi) in self
            .stages
            .iter()
            .enumerate()
            .take(self.stage_count as usize)
        {
            if *dpi < MIN_DPI || *dpi > MAX_DPI {
                return Err(DeviceError::invalid(format!(
                    "第 {} 档 DPI 必须在 {}–{} 之间",
                    i + 1,
                    MIN_DPI,
                    MAX_DPI
                )));
            }
            if *dpi % DPI_STEP != 0 {
                return Err(DeviceError::invalid(format!(
                    "第 {} 档 DPI 必须是 {} 的整数倍",
                    i + 1,
                    DPI_STEP
                )));
            }
        }
        if self.debounce_ms > 20 {
            return Err(DeviceError::invalid("防抖时间必须在 0–20 ms 之间"));
        }
        if self.sleep_minutes > 100 {
            return Err(DeviceError::invalid("休眠时间必须在 0–100 分钟之间"));
        }
        if !POLLING_TABLE.contains(&self.polling_hz) {
            return Err(DeviceError::invalid(format!(
                "不支持的回报率：{} Hz",
                self.polling_hz
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub key: String,
    pub kind: DeviceKind,
    pub model: Option<String>,
    pub product_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial: Option<String>,
    pub connection: ConnectionKind,
    pub supported: bool,
    pub capabilities: Capabilities,
    pub firmware: Option<String>,
    pub build_date: Option<String>,
    pub is_demo: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSummary {
    pub key: String,
    pub kind: DeviceKind,
    pub name: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub connection: ConnectionKind,
    pub supported: bool,
    pub status_note: Option<String>,
    pub is_demo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStatus {
    pub online: bool,
    pub battery: Option<Battery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub info: DeviceInfo,
    pub online: bool,
    pub battery: Option<Battery>,
    pub config: Option<MouseConfig>,
    pub buttons: Option<Vec<ButtonAction>>,
    /// 原厂默认值随快照一起给出，界面点「恢复默认」不需要再问一次设备。
    pub factory_config: MouseConfig,
    pub factory_buttons: Vec<ButtonAction>,
    pub read_at: u64,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn factory_default_is_valid() {
        MouseConfig::default().validate().unwrap();
    }

    #[test]
    fn stage_count_and_active_stage_are_checked() {
        let mut cfg = MouseConfig::default();
        cfg.stage_count = 0;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.stage_count = 7;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.stage_count = 3;
        cfg.active_stage = 4;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.active_stage = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn dpi_range_and_step_are_checked() {
        let mut cfg = MouseConfig::default();
        cfg.stages[0] = 50;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.stages[0] = 12_100;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.stages[0] = 850;
        assert!(cfg.validate().is_err());

        // 未启用的档位不参与校验
        let mut cfg = MouseConfig::default();
        cfg.stage_count = 1;
        cfg.active_stage = 1;
        cfg.stages[1] = 850;
        cfg.validate().unwrap();
    }

    #[test]
    fn debounce_sleep_and_polling_are_checked() {
        let mut cfg = MouseConfig::default();
        cfg.debounce_ms = 21;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.sleep_minutes = 101;
        assert!(cfg.validate().is_err());

        let mut cfg = MouseConfig::default();
        cfg.polling_hz = 1337;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn button_action_json_roundtrip() {
        for action in [
            ButtonAction::Left,
            ButtonAction::Right,
            ButtonAction::Middle,
            ButtonAction::Forward,
            ButtonAction::Backward,
            ButtonAction::Disabled,
            ButtonAction::Custom([0x21, 0x38, 0x01, 0x00]),
        ] {
            let json = serde_json::to_string(&action).unwrap();
            let parsed: ButtonAction = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, action);
        }
    }
}
