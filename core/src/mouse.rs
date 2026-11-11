use crate::a7::A7Device;
use crate::device::{scan_devices as scan_g3, Device};
use crate::error::DeviceError;
use crate::model::{ButtonAction, DeviceInfo, DeviceSummary, LiveStatus, MouseConfig, Snapshot};

/// 两只鼠标走两套完全不同的 HID 协议，但对界面暴露同样的快照。
pub enum MouseDevice {
    G3(Device),
    A7(A7Device),
}

pub fn scan(include_unknown: bool) -> Result<Vec<DeviceSummary>, DeviceError> {
    let mut list = scan_g3(include_unknown)?;
    // A7 会被 G3 扫描当成未知型号列出来；这里去重后交给 A7 扫描。
    list.retain(|d| !crate::a7::is_a7_pid(d.product_id));
    let mut a7 = crate::a7::scan()?;
    list.append(&mut a7);
    list.sort_by_key(|d| (d.kind as u8, !d.supported, d.name.clone()));
    Ok(list)
}

impl MouseDevice {
    pub fn open(key: &str) -> Result<Self, DeviceError> {
        if key.starts_with("a7:") {
            Ok(MouseDevice::A7(A7Device::open(key)?))
        } else {
            Ok(MouseDevice::G3(Device::open(key, false)?))
        }
    }

    pub fn info(&self) -> &DeviceInfo {
        match self {
            MouseDevice::G3(d) => &d.info,
            MouseDevice::A7(d) => &d.info,
        }
    }

    pub fn read_live(&mut self) -> Result<LiveStatus, DeviceError> {
        match self {
            MouseDevice::G3(d) => d.read_live(),
            MouseDevice::A7(d) => d.read_live(),
        }
    }

    pub fn snapshot(&mut self) -> Result<Snapshot, DeviceError> {
        match self {
            MouseDevice::G3(d) => d.snapshot(),
            MouseDevice::A7(d) => d.snapshot(),
        }
    }

    pub fn apply(
        &mut self,
        config: &MouseConfig,
        buttons: &[ButtonAction],
        saved: Option<&Snapshot>,
    ) -> Result<Snapshot, DeviceError> {
        match self {
            MouseDevice::G3(d) => {
                let config_changed = saved
                    .and_then(|s| s.config.as_ref())
                    .map(|c| c != config)
                    .unwrap_or(true);
                let buttons_changed = saved
                    .and_then(|s| s.buttons.as_ref())
                    .map(|b| b != buttons)
                    .unwrap_or(true);
                if config_changed {
                    d.set_config(config)?;
                }
                if buttons_changed && !buttons.is_empty() {
                    let mut arr = [ButtonAction::Left; 5];
                    if buttons.len() == 5 {
                        arr.copy_from_slice(buttons);
                        d.set_buttons(&arr)?;
                    }
                }
                d.snapshot()
            }
            MouseDevice::A7(d) => d.apply(config, buttons),
        }
    }
}
