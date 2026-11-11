use crate::model::{Capabilities, ConnectionKind, DpiRange};

/// MCHOSE 的 USB Vendor ID（有线鼠标与 2.4G 接收器共用）。
pub const MCHOSE_VENDOR_ID: u16 = 0x3837;

/// 厂商配置接口：Usage Page 0xFF01 / Usage 0x0010。
pub const CONFIG_USAGE_PAGE: u16 = 0xFF01;
pub const CONFIG_USAGE: u16 = 0x0010;

/// 目前经过实机验证的型号。
pub const SUPPORTED_MODELS: &[&str] = &["MCHOSE G3 V2"];

pub const PID_G3_V2_WIRED: u16 = 0x418C;
pub const PID_G3_V2_WIRELESS: u16 = 0x4245;

pub struct DeviceProfile {
    pub model: &'static str,
    pub display_name: &'static str,
    pub pids: &'static [u16],
    pub wireless_pids: &'static [u16],
    pub max_stages: u8,
    pub dpi_min: u16,
    pub dpi_max: u16,
    pub dpi_step: u16,
    pub polling_rates: &'static [u32],
}

pub static PROFILES: &[DeviceProfile] = &[DeviceProfile {
    model: "mchose-g3-v2",
    display_name: "MCHOSE G3 V2",
    pids: &[PID_G3_V2_WIRED, PID_G3_V2_WIRELESS],
    wireless_pids: &[PID_G3_V2_WIRELESS],
    max_stages: 6,
    dpi_min: 100,
    dpi_max: 12_000,
    dpi_step: 100,
    polling_rates: &[125, 250, 500, 1000],
}];

pub fn profile_for_pid(pid: u16) -> Option<&'static DeviceProfile> {
    PROFILES.iter().find(|p| p.pids.contains(&pid))
}

pub fn connection_for_pid(profile: &DeviceProfile, pid: u16) -> ConnectionKind {
    if profile.wireless_pids.contains(&pid) {
        ConnectionKind::Wireless
    } else {
        ConnectionKind::Wired
    }
}

pub fn capabilities_for(profile: &DeviceProfile) -> Capabilities {
    Capabilities {
        dpi: true,
        polling: true,
        debounce: true,
        sleep: !profile.wireless_pids.is_empty(),
        lod: true,
        buttons: true,
        scroll_direction: true,
        battery: true,
        firmware: true,
        dpi_range: DpiRange {
            min: profile.dpi_min,
            max: profile.dpi_max,
            step: profile.dpi_step,
        },
        polling_rates: profile.polling_rates.to_vec(),
        max_stages: profile.max_stages,
    }
}
