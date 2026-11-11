//! MHUB-Lite 的协议层。
//!
//! 这里刻意不依赖 Tauri：`cargo test -p mhub-core` 可以在没有窗口、没有设备、
//! 甚至没有外设的机器上跑完所有协议编解码测试。

mod a7;
mod catalog;
mod demo;
mod device;
mod error;
pub mod keyboard;
mod model;
mod mouse;
mod protocol;
mod transport;

pub use a7::{A7Device, A7Status};
pub use catalog::{
    capabilities_for, profile_for_pid, DeviceProfile, MCHOSE_VENDOR_ID, PROFILES, SUPPORTED_MODELS,
};
pub use demo::DemoTransport;
pub use device::{demo_summary, scan_devices, Device, HidDescriptor, DEMO_KEY};
pub use error::{DeviceError, ErrorCode};
pub use keyboard::{
    KeyboardDevice, KeyboardInput, KeyboardPerformance, KeyboardSnapshot, LightModeInfo,
};
pub use model::{
    Battery, ButtonAction, Capabilities, ConnectionKind, DeviceInfo, DeviceSummary, DpiRange,
    LiveStatus, Lod, MouseConfig, Snapshot, DEFAULT_BUTTONS, DPI_STEP, MAX_DPI, MIN_DPI,
};
pub use mouse::scan as scan_mice;
pub use mouse::MouseDevice;
pub use protocol::{
    build_get_buttons, build_get_config, build_set_buttons, build_set_config, parse_battery,
    parse_buttons, parse_config, parse_online, parse_version, CMD_BATTERY, CMD_GET_BUTTONS,
    CMD_GET_CONFIG, CMD_GET_VERSION, CMD_ONLINE, CMD_SET_BUTTONS, CMD_SET_CONFIG,
};
pub use transport::{HidTransport, Transport};
