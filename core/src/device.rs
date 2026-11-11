use crate::catalog::{
    capabilities_for, connection_for_pid, profile_for_pid, DeviceProfile, CONFIG_USAGE,
    CONFIG_USAGE_PAGE, MCHOSE_VENDOR_ID,
};
use crate::demo::DemoTransport;
use crate::error::{DeviceError, ErrorCode};
use crate::model::{
    Battery, ButtonAction, ConnectionKind, DeviceInfo, DeviceSummary, LiveStatus, MouseConfig,
    Snapshot, DEFAULT_BUTTONS, POLLING_TABLE,
};
use crate::protocol::{
    self, build_get_battery, build_get_buttons, build_get_config, build_get_online,
    build_get_version, build_set_buttons, build_set_config, CMD_BATTERY, CMD_GET_BUTTONS,
    CMD_GET_CONFIG, CMD_GET_VERSION, CMD_ONLINE, CMD_SET_BUTTONS, CMD_SET_CONFIG,
};
use crate::transport::{HidTransport, Transport};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEMO_KEY: &str = "demo";

#[derive(Clone)]
pub struct HidDescriptor {
    pub key: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial: Option<String>,
    pub product_name: Option<String>,
    pub profile: Option<&'static DeviceProfile>,
    pub has_config_interface: bool,
}

pub fn scan_devices(include_unknown: bool) -> Result<Vec<DeviceSummary>, DeviceError> {
    let api = hidapi::HidApi::new()?;
    let mut found: Vec<HidDescriptor> = Vec::new();

    for info in api.device_list() {
        if info.vendor_id() != MCHOSE_VENDOR_ID {
            continue;
        }
        let pid = info.product_id();
        let profile = profile_for_pid(pid);
        if profile.is_none() && !include_unknown {
            continue;
        }
        let is_config = info.usage_page() == CONFIG_USAGE_PAGE && info.usage() == CONFIG_USAGE;
        let serial = info.serial_number().map(|s| s.to_string());
        let key = format!(
            "{:04x}:{:04x}:{}",
            MCHOSE_VENDOR_ID,
            pid,
            serial.as_deref().unwrap_or("")
        );

        if let Some(existing) = found.iter_mut().find(|d| d.key == key) {
            // 同一台设备会暴露多个 HID 接口，只保留厂商配置接口那一条。
            if is_config && !existing.has_config_interface {
                existing.has_config_interface = true;
                existing.product_name = info.product_string().map(|s| s.to_string());
            }
            continue;
        }

        found.push(HidDescriptor {
            key,
            vendor_id: MCHOSE_VENDOR_ID,
            product_id: pid,
            serial,
            product_name: info.product_string().map(|s| s.to_string()),
            profile,
            has_config_interface: is_config,
        });
    }

    let mut summaries: Vec<DeviceSummary> = found
        .into_iter()
        .map(|d| {
            let supported = d.profile.is_some() && d.has_config_interface;
            let name = d
                .profile
                .map(|p| p.display_name.to_string())
                .or_else(|| d.product_name.clone())
                .unwrap_or_else(|| format!("MCHOSE 设备 (0x{:04X})", d.product_id));
            let connection = d
                .profile
                .map(|p| connection_for_pid(p, d.product_id))
                .unwrap_or(ConnectionKind::Unknown);
            let status_note = if d.profile.is_none() {
                Some("这个型号还没有经过实机验证的配置协议，当前只显示识别信息。".to_string())
            } else if !d.has_config_interface {
                Some("没有找到厂商配置接口（0xFF01），可能被其他程序独占。".to_string())
            } else {
                None
            };
            DeviceSummary {
                key: d.key,
                kind: crate::model::DeviceKind::Mouse,
                name,
                model: d.profile.map(|p| p.model.to_string()),
                serial: d.serial,
                vendor_id: d.vendor_id,
                product_id: d.product_id,
                connection,
                supported,
                status_note,
                is_demo: false,
            }
        })
        .collect();

    // 能用的排前面，无线排在后面（同时插着线时优先有线）。
    summaries.sort_by_key(|s| (!s.supported, s.connection == ConnectionKind::Wireless));
    Ok(summaries)
}

pub fn demo_summary() -> DeviceSummary {
    DeviceSummary {
        key: DEMO_KEY.to_string(),
        kind: crate::model::DeviceKind::Mouse,
        name: "MHUB 演示设备".to_string(),
        model: Some("mhub-demo".to_string()),
        serial: Some("DEMO-0001".to_string()),
        vendor_id: MCHOSE_VENDOR_ID,
        product_id: 0x4245,
        connection: ConnectionKind::Wireless,
        supported: true,
        status_note: Some("演示模式：写入只作用于本机模拟设备，不会碰真实硬件。".to_string()),
        is_demo: true,
    }
}

pub struct Device {
    transport: Box<dyn Transport>,
    pub info: DeviceInfo,
}

impl Device {
    pub fn open(key: &str, demo: bool) -> Result<Self, DeviceError> {
        if demo && key == DEMO_KEY {
            return Ok(Self::demo());
        }

        let descriptors = scan_all()?;
        let desc = descriptors
            .into_iter()
            .find(|d| d.key == key)
            .ok_or_else(|| {
                DeviceError::not_found("设备已断开或不在线：请重新插拔，或移动鼠标唤醒")
            })?;

        let profile = desc.profile.ok_or_else(|| {
            DeviceError::unsupported(
                "这个 MCHOSE 型号还没有验证过的配置协议，暂不支持读写，避免误写坏设备。",
            )
        })?;
        if !desc.has_config_interface {
            return Err(DeviceError::unsupported(
                "没有找到厂商配置接口（Usage Page 0xFF01），无法读写设备。",
            ));
        }

        let transport =
            HidTransport::open(desc.vendor_id, desc.product_id, desc.serial.as_deref())?;

        let mut info = DeviceInfo {
            key: desc.key.clone(),
            kind: crate::model::DeviceKind::Mouse,
            model: Some(profile.model.to_string()),
            product_name: desc
                .product_name
                .clone()
                .unwrap_or_else(|| profile.display_name.to_string()),
            vendor_id: desc.vendor_id,
            product_id: desc.product_id,
            serial: desc.serial.clone(),
            connection: connection_for_pid(profile, desc.product_id),
            supported: true,
            capabilities: capabilities_for(profile),
            firmware: None,
            build_date: None,
            is_demo: false,
            note: None,
        };

        let mut device = Self {
            transport: Box::new(transport),
            info: info.clone(),
        };

        // 固件版本只在连接时读一次；无线休眠时读不到也没关系。
        if device.get_online().unwrap_or(false) {
            if let Ok((fw, date)) = device.query_version() {
                info.firmware = Some(fw);
                info.build_date = if date.is_empty() { None } else { Some(date) };
                device.info = info;
            }
        }

        Ok(device)
    }

    fn demo() -> Self {
        let profile = &crate::catalog::PROFILES[0];
        let info = DeviceInfo {
            key: DEMO_KEY.to_string(),
            kind: crate::model::DeviceKind::Mouse,
            model: Some("mhub-demo".to_string()),
            product_name: "MHUB 演示设备".to_string(),
            vendor_id: MCHOSE_VENDOR_ID,
            product_id: 0x4245,
            serial: Some("DEMO-0001".to_string()),
            connection: ConnectionKind::Wireless,
            supported: true,
            capabilities: capabilities_for(profile),
            firmware: Some("MHUB-DEMO 0.1.0".to_string()),
            build_date: Some("202611".to_string()),
            is_demo: true,
            note: Some("演示模式：写入只作用于本机模拟设备，不会碰真实硬件。".to_string()),
        };
        Self {
            transport: Box::new(DemoTransport::new()),
            info,
        }
    }

    #[cfg(test)]
    pub fn open_demo_for_test() -> Self {
        Self::demo()
    }

    pub fn read_live(&mut self) -> Result<LiveStatus, DeviceError> {
        let online = self.get_online().unwrap_or(false);
        let battery = if online {
            self.get_battery().ok()
        } else {
            None
        };
        Ok(LiveStatus { online, battery })
    }

    pub fn snapshot(&mut self) -> Result<Snapshot, DeviceError> {
        let online = self.get_online().unwrap_or(false);
        let battery = if online {
            self.get_battery().ok()
        } else {
            None
        };

        let mut config = None;
        let mut buttons = None;
        if online {
            config = Some(self.get_config()?);
            buttons = self.get_buttons().ok().map(|b| b.to_vec());
        }

        Ok(Snapshot {
            info: self.info.clone(),
            online,
            battery,
            config,
            buttons,
            factory_config: MouseConfig::default(),
            factory_buttons: DEFAULT_BUTTONS.to_vec(),
            read_at: unix_now(),
        })
    }

    pub fn query_version(&mut self) -> Result<(String, String), DeviceError> {
        let mut last = DeviceError::timeout("读取固件版本超时");
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(40));
            }
            if self.transport.write_packet(&build_get_version()).is_err() {
                continue;
            }
            match self.read_response(CMD_GET_VERSION, 150) {
                Ok(pkt) => match protocol::parse_version(&pkt) {
                    Ok(v) => return Ok(v),
                    Err(e) => last = e,
                },
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn get_online(&mut self) -> Result<bool, DeviceError> {
        if self.info.connection == ConnectionKind::Wired {
            return Ok(true);
        }

        let mut last = DeviceError::timeout("查询在线状态超时");
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(40));
            }
            if self.transport.write_packet(&build_get_online()).is_err() {
                continue;
            }
            match self.read_response(CMD_ONLINE, 150) {
                Ok(pkt) => return protocol::parse_online(&pkt),
                Err(e) => {
                    // 休眠时收不到回包是正常的；权限/设备错误要继续往上报。
                    if matches!(e.code, ErrorCode::Permission | ErrorCode::Hid) {
                        return Err(e);
                    }
                    last = e;
                }
            }
        }
        // 无线接收器在但鼠标睡着：返回 false，而不是假装设备不存在。
        let _ = last;
        Ok(false)
    }

    pub fn get_battery(&mut self) -> Result<Battery, DeviceError> {
        let mut last = DeviceError::timeout("读取电量超时");
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(40));
            }
            if self.transport.write_packet(&build_get_battery()).is_err() {
                continue;
            }
            match self.read_response(CMD_BATTERY, 150) {
                Ok(pkt) => match protocol::parse_battery(&pkt) {
                    Ok((percent, charging)) => return Ok(Battery { percent, charging }),
                    Err(e) => last = e,
                },
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn get_config(&mut self) -> Result<MouseConfig, DeviceError> {
        let mut last = DeviceError::timeout("读取配置超时");
        for attempt in 0..4 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(60));
            }
            if self.transport.write_packet(&build_get_config()).is_err() {
                continue;
            }
            match self.read_response(CMD_GET_CONFIG, 150) {
                Ok(pkt) => match protocol::parse_config(&pkt) {
                    Ok(cfg) => return Ok(cfg),
                    Err(e) => last = e,
                },
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn get_buttons(&mut self) -> Result<[ButtonAction; 5], DeviceError> {
        let mut last = DeviceError::timeout("读取按键映射超时");
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(40));
            }
            if self.transport.write_packet(&build_get_buttons()).is_err() {
                continue;
            }
            match self.read_response(CMD_GET_BUTTONS, 200) {
                Ok(pkt) => match protocol::parse_buttons(&pkt) {
                    Ok(buttons) => return Ok(buttons),
                    Err(e) => last = e,
                },
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn set_config(&mut self, cfg: &MouseConfig) -> Result<(), DeviceError> {
        cfg.validate()?;
        let caps = &self.info.capabilities;
        if !caps.polling_rates.contains(&cfg.polling_hz) {
            return Err(DeviceError::invalid(format!(
                "这个型号不支持 {} Hz 回报率",
                cfg.polling_hz
            )));
        }
        if cfg.stage_count > caps.max_stages {
            return Err(DeviceError::invalid(format!(
                "这个型号最多支持 {} 个 DPI 档位",
                caps.max_stages
            )));
        }
        for (i, dpi) in cfg.stages.iter().enumerate().take(cfg.stage_count as usize) {
            let range = caps.dpi_range;
            if *dpi < range.min || *dpi > range.max || *dpi % range.step != 0 {
                return Err(DeviceError::invalid(format!(
                    "第 {} 档 DPI 必须是 {}–{} 之间、以 {} 为步进的值",
                    i + 1,
                    range.min,
                    range.max,
                    range.step
                )));
            }
        }
        if !self.get_online()? {
            return Err(DeviceError::offline(
                "鼠标正在休眠：先移动一下鼠标或按任意键唤醒，再写入配置",
            ));
        }

        let packet = build_set_config(cfg);
        let mut last = DeviceError::timeout("写入配置超时");
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(40));
            }
            if self.transport.write_packet(&packet).is_err() {
                continue;
            }
            match self.read_response(CMD_SET_CONFIG, 150) {
                Ok(_) => {
                    std::thread::sleep(Duration::from_millis(30));
                    return Ok(());
                }
                Err(e) => last = e,
            }
        }
        Err(DeviceError::protocol(format!(
            "写入配置后没有收到设备确认：{}",
            last.message
        )))
    }

    pub fn set_buttons(&mut self, buttons: &[ButtonAction; 5]) -> Result<(), DeviceError> {
        if !self.get_online()? {
            return Err(DeviceError::offline(
                "鼠标正在休眠：先移动一下鼠标或按任意键唤醒，再写入按键",
            ));
        }

        let packet = build_set_buttons(buttons);
        let mut last = DeviceError::timeout("写入按键映射超时");
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(40));
            }
            if self.transport.write_packet(&packet).is_err() {
                continue;
            }
            match self.read_response(CMD_SET_BUTTONS, 200) {
                Ok(_) => {
                    // 写按键会让固件刷新 HID 描述符，鼠标/键盘端点会短暂消失。
                    std::thread::sleep(Duration::from_millis(250));
                    return Ok(());
                }
                Err(e) => last = e,
            }
        }
        Err(DeviceError::protocol(format!(
            "写入按键映射后没有收到设备确认：{}",
            last.message
        )))
    }

    pub fn supported_polling_rates(&self) -> Vec<u32> {
        self.info.capabilities.polling_rates.clone()
    }

    pub fn polling_table(&self) -> &'static [u32] {
        &POLLING_TABLE
    }

    fn read_response(&mut self, cmd: u8, timeout_ms: u64) -> Result<[u8; 64], DeviceError> {
        let start = Instant::now();
        let limit = Duration::from_millis(timeout_ms);
        loop {
            let elapsed = start.elapsed();
            if elapsed >= limit {
                return Err(DeviceError::timeout(format!(
                    "等待命令 0x{cmd:02X} 的回应超时"
                )));
            }
            let remaining = (limit - elapsed).as_millis().max(1) as i32;
            let pkt = self.transport.read_packet(remaining)?;
            if pkt[0] == 0xAA && pkt[1] == cmd {
                return Ok(pkt);
            }
        }
    }
}

fn scan_all() -> Result<Vec<HidDescriptor>, DeviceError> {
    let api = hidapi::HidApi::new()?;
    let mut found: Vec<HidDescriptor> = Vec::new();
    for info in api.device_list() {
        if info.vendor_id() != MCHOSE_VENDOR_ID {
            continue;
        }
        let pid = info.product_id();
        let serial = info.serial_number().map(|s| s.to_string());
        let key = format!(
            "{:04x}:{:04x}:{}",
            MCHOSE_VENDOR_ID,
            pid,
            serial.as_deref().unwrap_or("")
        );
        let is_config = info.usage_page() == CONFIG_USAGE_PAGE && info.usage() == CONFIG_USAGE;

        if let Some(existing) = found.iter_mut().find(|d| d.key == key) {
            if is_config {
                existing.has_config_interface = true;
            }
            continue;
        }

        found.push(HidDescriptor {
            key,
            vendor_id: MCHOSE_VENDOR_ID,
            product_id: pid,
            serial,
            product_name: info.product_string().map(|s| s.to_string()),
            profile: profile_for_pid(pid),
            has_config_interface: is_config,
        });
    }
    Ok(found)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
