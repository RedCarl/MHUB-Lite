//! 演示设备：一个完全按 G3 V2 帧格式应答的内存外设。
//!
//! 用途有两个：
//! 1. 没有鼠标的机器上能走完整的界面流程（读取 → 修改 → 写入 → 回读）；
//! 2. 协议层的集成测试不需要真机。

use crate::error::DeviceError;
use crate::model::{ButtonAction, MouseConfig, DEFAULT_BUTTONS};
use crate::protocol::{
    self, CMD_BATTERY, CMD_GET_BUTTONS, CMD_GET_CONFIG, CMD_GET_VERSION, CMD_ONLINE,
    CMD_SET_BUTTONS, CMD_SET_CONFIG,
};
use crate::transport::Transport;
use std::collections::VecDeque;

pub struct DemoTransport {
    config: MouseConfig,
    buttons: [ButtonAction; 5],
    battery: u8,
    charging: bool,
    queue: VecDeque<[u8; 64]>,
}

impl DemoTransport {
    pub fn new() -> Self {
        Self {
            config: MouseConfig::default(),
            buttons: DEFAULT_BUTTONS,
            battery: 82,
            charging: false,
            queue: VecDeque::new(),
        }
    }

    fn respond(&mut self, pkt: [u8; 64]) {
        self.queue.push_back(pkt);
    }

    fn ack(cmd: u8) -> [u8; 64] {
        let mut pkt = [0u8; 64];
        pkt[0] = 0xAA;
        pkt[1] = cmd;
        pkt[2] = 0xA5;
        pkt[3] = 0x41;
        pkt
    }
}

impl Default for DemoTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for DemoTransport {
    fn write_packet(&mut self, payload: &[u8]) -> Result<(), DeviceError> {
        if payload.len() < 2 {
            return Err(DeviceError::protocol("演示设备收到了空包"));
        }
        let cmd = payload[1];
        match cmd {
            CMD_GET_VERSION => {
                let mut pkt = [0u8; 64];
                pkt[0] = 0xAA;
                pkt[1] = CMD_GET_VERSION;
                pkt[2] = 0xA5;
                let fw = b"MHUB-DEMO 0.1.0";
                pkt[8..8 + fw.len()].copy_from_slice(fw);
                pkt[26..32].copy_from_slice(b"202611");
                self.respond(pkt);
            }
            CMD_GET_CONFIG => {
                let pkt = protocol::encode_config_response(&self.config);
                self.respond(pkt);
            }
            CMD_SET_CONFIG => {
                let mut full = [0u8; 64];
                let n = payload.len().min(64);
                full[..n].copy_from_slice(&payload[..n]);
                self.config = protocol::parse_set_config_request(&full);
                self.respond(Self::ack(CMD_SET_CONFIG));
            }
            CMD_GET_BUTTONS => {
                let pkt = protocol::encode_buttons_response(&self.buttons);
                self.respond(pkt);
            }
            CMD_SET_BUTTONS => {
                let mut full = [0u8; 64];
                let n = payload.len().min(64);
                full[..n].copy_from_slice(&payload[..n]);
                self.buttons = protocol::parse_set_buttons_request(&full);
                self.respond(Self::ack(CMD_SET_BUTTONS));
            }
            CMD_BATTERY => {
                let mut pkt = [0u8; 64];
                pkt[0] = 0xAA;
                pkt[1] = CMD_BATTERY;
                pkt[2] = 0xA5;
                pkt[3] = 0x0B;
                pkt[4] = 0x2E;
                pkt[8] = self.battery;
                pkt[9] = u8::from(self.charging);
                self.respond(pkt);
            }
            CMD_ONLINE => {
                let mut pkt = [0u8; 64];
                pkt[0] = 0xAA;
                pkt[1] = CMD_ONLINE;
                pkt[2] = 0xA5;
                pkt[8] = 1;
                self.respond(pkt);
            }
            _ => {
                // 未知命令保持沉默，上层会按超时处理 —— 和真机一致。
            }
        }
        Ok(())
    }

    fn read_packet(&mut self, _timeout_ms: i32) -> Result<[u8; 64], DeviceError> {
        self.queue
            .pop_front()
            .ok_or_else(|| DeviceError::timeout("演示设备没有更多回应"))
    }

    fn flush_input(&mut self) {
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::Device;

    #[test]
    fn demo_device_roundtrip() {
        let mut device = Device::open_demo_for_test();
        let snapshot = device.snapshot().unwrap();
        assert!(snapshot.online);
        assert_eq!(snapshot.battery.unwrap().percent, 82);
        let mut cfg = snapshot.config.unwrap();
        assert_eq!(cfg.stages[0], 400);

        cfg.polling_hz = 1000;
        cfg.stages[0] = 800;
        cfg.stage_count = 4;
        cfg.active_stage = 2;
        cfg.debounce_ms = 4;
        cfg.sleep_minutes = 0;
        cfg.scroll_reversed = true;
        device.set_config(&cfg).unwrap();

        let buttons = [
            ButtonAction::Left,
            ButtonAction::Middle,
            ButtonAction::Right,
            ButtonAction::Backward,
            ButtonAction::Forward,
        ];
        device.set_buttons(&buttons).unwrap();

        let after = device.snapshot().unwrap();
        assert_eq!(after.config.unwrap(), cfg);
        assert_eq!(after.buttons.unwrap(), buttons);
    }
}
