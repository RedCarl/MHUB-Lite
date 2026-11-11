//! G3 V2 的二进制帧格式。
//!
//! 请求帧固定以 `0x55` 开头、`0xA5` 作标记；回应以 `0xAA` 开头。
//! 每一帧 64 字节，HID 写入时前面再加一个 Report ID（0x00），共 65 字节。
//!
//! 这里只做纯字节运算，不接触设备 —— 协议测试因此不需要外设。

use crate::error::DeviceError;
use crate::model::{ButtonAction, Lod, MouseConfig, POLLING_TABLE};

pub const CMD_GET_VERSION: u8 = 0x03;
pub const CMD_GET_BUTTONS: u8 = 0x08;
pub const CMD_SET_BUTTONS: u8 = 0x09;
pub const CMD_GET_CONFIG: u8 = 0x0E;
pub const CMD_SET_CONFIG: u8 = 0x0F;
pub const CMD_BATTERY: u8 = 0x30;
pub const CMD_ONLINE: u8 = 0xED;

const MAGIC_REQ: u8 = 0x55;
const MAGIC_MARK: u8 = 0xA5;
const MAGIC_RESP: u8 = 0xAA;

pub fn build_get_version() -> [u8; 2] {
    [MAGIC_REQ, CMD_GET_VERSION]
}

pub fn build_get_config() -> [u8; 10] {
    [
        MAGIC_REQ,
        CMD_GET_CONFIG,
        MAGIC_MARK,
        0x0B,
        0x30,
        1,
        1,
        1,
        0,
        0,
    ]
}

pub fn build_set_config(cfg: &MouseConfig) -> [u8; 64] {
    let mut pkt = [0u8; 64];
    pkt[0] = MAGIC_REQ;
    pkt[1] = CMD_SET_CONFIG;
    pkt[2] = 0xAE;
    pkt[3] = 0x0A;
    pkt[4] = 0x30;
    pkt[5] = 1;
    pkt[6] = 1;
    pkt[7] = 1;
    pkt[8] = 0;
    pkt[9] = 0;

    let stage_count = cfg.stage_count.clamp(1, 6);
    let active = cfg.active_stage.clamp(1, stage_count);
    pkt[10] = polling_index(cfg.polling_hz).unwrap_or(2) + 1;
    pkt[11] = stage_count;
    pkt[12] = active;

    for (i, dpi) in cfg.stages.iter().enumerate() {
        let offset = 13 + i * 2;
        pkt[offset] = (dpi & 0xFF) as u8;
        pkt[offset + 1] = (dpi >> 8) as u8;
    }

    pkt[48] = u8::from(cfg.scroll_reversed);
    pkt[49] = cfg.lod.to_u8();
    pkt[50] = cfg.sensor_flags;
    pkt[51] = cfg.debounce_ms;
    pkt[52] = cfg.sleep_minutes;
    pkt[53] = cfg.highspeed_mode;
    pkt[55] = cfg.sensor_angle as u8;
    pkt
}

pub fn parse_config(pkt: &[u8; 64]) -> Result<MouseConfig, DeviceError> {
    check_response(pkt, CMD_GET_CONFIG)?;

    let dpi_stage1 = u16::from_le_bytes([pkt[13], pkt[14]]);
    let uninitialized = (pkt[13] == 0 && pkt[14] == 0 && pkt[15] == 0)
        || (pkt[13] == 0xFF && pkt[14] == 0xFF && pkt[15] == 0xFF)
        || pkt[11] == 0
        || dpi_stage1 < 50;
    if uninitialized {
        return Err(DeviceError::offline(
            "设备没有返回有效配置（可能正在休眠，或接收器尚未连上鼠标）",
        ));
    }

    let stage_count = pkt[11].clamp(1, 6);
    let active_stage = (pkt[12] as usize).clamp(1, stage_count as usize) as u8;

    let mut stages = [400, 800, 1600, 3200, 6400, 12_000];
    for (i, stage) in stages.iter_mut().enumerate() {
        let offset = 13 + i * 2;
        let val = u16::from_le_bytes([pkt[offset], pkt[offset + 1]]);
        if (50..=50_000).contains(&val) {
            *stage = val;
        }
    }

    let polling_hz = POLLING_TABLE
        .get(pkt[10].saturating_sub(1) as usize)
        .copied()
        .unwrap_or(1000);

    Ok(MouseConfig {
        polling_hz,
        stage_count,
        active_stage,
        stages,
        lod: Lod::from_u8(pkt[49]),
        debounce_ms: pkt[51],
        sleep_minutes: pkt[52],
        scroll_reversed: pkt[48] != 0,
        sensor_flags: pkt[50],
        highspeed_mode: pkt[53],
        sensor_angle: pkt[55] as i8,
    })
}

pub fn build_get_buttons() -> [u8; 10] {
    [
        MAGIC_REQ,
        CMD_GET_BUTTONS,
        MAGIC_MARK,
        0x0B,
        0x20,
        0,
        0,
        0,
        0,
        0,
    ]
}

/// 五个可改按键对应硬件槽位：0 左键（锁定）、1 右键、2 中键、3 后退、4 前进。
///
/// 固件把槽位 3 给后退、槽位 4 给前进；界面顺序是
/// `[左, 右, 中, 前进, 后退]`，所以写包时 3 和 4 要交换。
pub fn build_set_buttons(buttons: &[ButtonAction; 5]) -> [u8; 64] {
    let mut pkt = [0u8; 64];
    pkt[0] = MAGIC_REQ;
    pkt[1] = CMD_SET_BUTTONS;
    pkt[2] = MAGIC_MARK;
    pkt[3] = 0x22;
    pkt[4] = 0x20;

    // 左键槽位始终写左键：固件本身忽略这里的改动，官方驱动也锁死。
    pkt[8..12].copy_from_slice(&ButtonAction::Left.to_wire());
    pkt[12..16].copy_from_slice(&buttons[1].to_wire());
    pkt[16..20].copy_from_slice(&buttons[2].to_wire());
    pkt[20..24].copy_from_slice(&buttons[4].to_wire());
    pkt[24..28].copy_from_slice(&buttons[3].to_wire());

    // 槽位 5–7 不参与改键，按固件默认值写回：DPI 键、滚轮上、滚轮下。
    pkt[28..32].copy_from_slice(&[0x21, 0x55, 0x00, 0x00]);
    pkt[32..36].copy_from_slice(&[0x21, 0x38, 0x01, 0x00]);
    pkt[36..40].copy_from_slice(&[0x21, 0x38, 0xFF, 0x00]);
    pkt
}

pub fn parse_buttons(pkt: &[u8; 64]) -> Result<[ButtonAction; 5], DeviceError> {
    check_response(pkt, CMD_GET_BUTTONS)?;

    let slot = |offset: usize, default: ButtonAction| -> ButtonAction {
        let raw = [
            pkt[offset],
            pkt[offset + 1],
            pkt[offset + 2],
            pkt[offset + 3],
        ];
        if raw == [0, 0, 0, 0] {
            default
        } else {
            ButtonAction::from_wire(raw)
        }
    };

    Ok([
        ButtonAction::Left,
        slot(12, ButtonAction::Right),
        slot(16, ButtonAction::Middle),
        slot(24, ButtonAction::Forward),
        slot(20, ButtonAction::Backward),
    ])
}

pub fn build_get_battery() -> [u8; 10] {
    [
        MAGIC_REQ,
        CMD_BATTERY,
        MAGIC_MARK,
        0x0B,
        0x2E,
        1,
        1,
        1,
        0,
        0,
    ]
}

pub fn parse_battery(pkt: &[u8; 64]) -> Result<(u8, bool), DeviceError> {
    check_response(pkt, CMD_BATTERY)?;
    let percent = pkt[8];
    if !(1..=100).contains(&percent) {
        return Err(DeviceError::protocol(format!(
            "电量读数不在 1–100 之间（收到 {percent}）"
        )));
    }
    Ok((percent, pkt[9] != 0))
}

pub fn build_get_online() -> [u8; 2] {
    [MAGIC_REQ, CMD_ONLINE]
}

pub fn parse_online(pkt: &[u8; 64]) -> Result<bool, DeviceError> {
    check_response(pkt, CMD_ONLINE)?;
    Ok(pkt[8] != 0)
}

pub fn parse_version(pkt: &[u8; 64]) -> Result<(String, String), DeviceError> {
    check_response(pkt, CMD_GET_VERSION)?;
    let firmware = fixed_string(&pkt[8..23]);
    let build_date = fixed_string(&pkt[26..32]);
    if firmware.is_empty() {
        return Err(DeviceError::protocol("固件版本为空"));
    }
    Ok((firmware, build_date))
}

fn fixed_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_matches('\0')
        .trim()
        .to_string()
}

fn check_response(pkt: &[u8; 64], cmd: u8) -> Result<(), DeviceError> {
    if pkt[0] != MAGIC_RESP || pkt[1] != cmd {
        return Err(DeviceError::protocol(format!(
            "回应帧不匹配：期望 AA {:02X}，收到 {:02X} {:02X}",
            cmd, pkt[0], pkt[1]
        )));
    }
    Ok(())
}

pub fn polling_index(hz: u32) -> Option<u8> {
    POLLING_TABLE.iter().position(|v| *v == hz).map(|i| i as u8)
}

/// 供演示设备与测试使用的「读配置」回应帧，格式与真机一致。
pub(crate) fn encode_config_response(cfg: &MouseConfig) -> [u8; 64] {
    let mut pkt = [0u8; 64];
    pkt[0] = MAGIC_RESP;
    pkt[1] = CMD_GET_CONFIG;
    pkt[2] = MAGIC_MARK;
    pkt[3] = 0x0B;
    pkt[4] = 0x30;
    pkt[5] = 1;
    pkt[6] = 1;
    pkt[7] = 1;
    pkt[10] = polling_index(cfg.polling_hz).unwrap_or(2) + 1;
    pkt[11] = cfg.stage_count.clamp(1, 6);
    pkt[12] = cfg.active_stage.clamp(1, cfg.stage_count.clamp(1, 6));
    for (i, dpi) in cfg.stages.iter().enumerate() {
        let offset = 13 + i * 2;
        pkt[offset] = (dpi & 0xFF) as u8;
        pkt[offset + 1] = (dpi >> 8) as u8;
    }
    pkt[48] = u8::from(cfg.scroll_reversed);
    pkt[49] = cfg.lod.to_u8();
    pkt[50] = cfg.sensor_flags;
    pkt[51] = cfg.debounce_ms;
    pkt[52] = cfg.sleep_minutes;
    pkt[53] = cfg.highspeed_mode;
    pkt[55] = cfg.sensor_angle as u8;
    pkt
}

pub(crate) fn encode_buttons_response(buttons: &[ButtonAction; 5]) -> [u8; 64] {
    let mut pkt = [0u8; 64];
    pkt[0] = MAGIC_RESP;
    pkt[1] = CMD_GET_BUTTONS;
    pkt[2] = MAGIC_MARK;
    pkt[3] = 0x0B;
    pkt[4] = 0x20;
    pkt[8..12].copy_from_slice(&ButtonAction::Left.to_wire());
    pkt[12..16].copy_from_slice(&buttons[1].to_wire());
    pkt[16..20].copy_from_slice(&buttons[2].to_wire());
    pkt[20..24].copy_from_slice(&buttons[4].to_wire());
    pkt[24..28].copy_from_slice(&buttons[3].to_wire());
    pkt[28..32].copy_from_slice(&[0x21, 0x55, 0x00, 0x00]);
    pkt[32..36].copy_from_slice(&[0x21, 0x38, 0x01, 0x00]);
    pkt[36..40].copy_from_slice(&[0x21, 0x38, 0xFF, 0x00]);
    pkt
}

/// 解析「写配置」请求帧（演示设备用；真机只写不读这条）。
pub(crate) fn parse_set_config_request(pkt: &[u8; 64]) -> MouseConfig {
    let stage_count = pkt[11].clamp(1, 6);
    let active_stage = (pkt[12] as usize).clamp(1, stage_count as usize) as u8;
    let mut stages = [400, 800, 1600, 3200, 6400, 12_000];
    for (i, stage) in stages.iter_mut().enumerate() {
        let offset = 13 + i * 2;
        let val = u16::from_le_bytes([pkt[offset], pkt[offset + 1]]);
        if (50..=50_000).contains(&val) {
            *stage = val;
        }
    }
    MouseConfig {
        polling_hz: POLLING_TABLE
            .get(pkt[10].saturating_sub(1) as usize)
            .copied()
            .unwrap_or(1000),
        stage_count,
        active_stage,
        stages,
        lod: Lod::from_u8(pkt[49]),
        debounce_ms: pkt[51],
        sleep_minutes: pkt[52],
        scroll_reversed: pkt[48] != 0,
        sensor_flags: pkt[50],
        highspeed_mode: pkt[53],
        sensor_angle: pkt[55] as i8,
    }
}

/// 解析「写按键」请求帧（演示设备用）。
pub(crate) fn parse_set_buttons_request(pkt: &[u8; 64]) -> [ButtonAction; 5] {
    let slot = |offset: usize, default: ButtonAction| -> ButtonAction {
        let raw = [
            pkt[offset],
            pkt[offset + 1],
            pkt[offset + 2],
            pkt[offset + 3],
        ];
        if raw == [0, 0, 0, 0] {
            default
        } else {
            ButtonAction::from_wire(raw)
        }
    };
    [
        ButtonAction::Left,
        slot(12, ButtonAction::Right),
        slot(16, ButtonAction::Middle),
        slot(24, ButtonAction::Forward),
        slot(20, ButtonAction::Backward),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MouseConfig, DEFAULT_BUTTONS};

    #[test]
    fn set_config_packet_has_expected_header_and_payload() {
        let cfg = MouseConfig {
            polling_hz: 1000,
            stage_count: 3,
            active_stage: 2,
            stages: [400, 800, 1600, 3200, 6400, 12_000],
            debounce_ms: 4,
            sleep_minutes: 0,
            scroll_reversed: true,
            lod: Lod::Low,
            ..Default::default()
        };

        let pkt = build_set_config(&cfg);
        assert_eq!(&pkt[0..10], &[0x55, 0x0F, 0xAE, 0x0A, 0x30, 1, 1, 1, 0, 0]);
        assert_eq!(pkt[10], 4, "1000Hz 在协议里是索引 3 + 1");
        assert_eq!(pkt[11], 3);
        assert_eq!(pkt[12], 2);
        assert_eq!(u16::from_le_bytes([pkt[13], pkt[14]]), 400);
        assert_eq!(u16::from_le_bytes([pkt[15], pkt[16]]), 800);
        assert_eq!(u16::from_le_bytes([pkt[17], pkt[18]]), 1600);
        assert_eq!(pkt[48], 1);
        assert_eq!(pkt[49], 0);
        assert_eq!(pkt[51], 4);
        assert_eq!(pkt[52], 0);
    }

    #[test]
    fn set_config_clamps_invalid_stage_numbers() {
        let mut cfg = MouseConfig {
            stage_count: 2,
            active_stage: 5,
            ..Default::default()
        };
        let pkt = build_set_config(&cfg);
        assert_eq!(pkt[11], 2);
        assert_eq!(pkt[12], 2);

        cfg.stage_count = 0;
        cfg.active_stage = 0;
        let pkt = build_set_config(&cfg);
        assert_eq!(pkt[11], 1);
        assert_eq!(pkt[12], 1);
    }

    #[test]
    fn config_roundtrip_preserves_opaque_fields() {
        let cfg = MouseConfig {
            sensor_flags: 7,
            highspeed_mode: 2,
            sensor_angle: -3,
            scroll_reversed: true,
            lod: Lod::Low,
            polling_hz: 500,
            ..Default::default()
        };

        let resp = encode_config_response(&cfg);
        let parsed = parse_config(&resp).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn config_parse_rejects_uninitialized_response() {
        let mut pkt = [0u8; 64];
        pkt[0] = 0xAA;
        pkt[1] = CMD_GET_CONFIG;
        pkt[11] = 0;
        let err = parse_config(&pkt).unwrap_err();
        assert_eq!(err.code, crate::ErrorCode::Offline);
    }

    #[test]
    fn set_buttons_packet_swaps_side_buttons_and_locks_left() {
        let buttons = [
            ButtonAction::Disabled,
            ButtonAction::Right,
            ButtonAction::Middle,
            ButtonAction::Backward,
            ButtonAction::Forward,
        ];
        let pkt = build_set_buttons(&buttons);
        assert_eq!(&pkt[0..5], &[0x55, 0x09, 0xA5, 0x22, 0x20]);
        assert_eq!(&pkt[8..12], &[0x20, 0x01, 0x00, 0x00], "左键必须锁定");
        assert_eq!(&pkt[12..16], &buttons[1].to_wire());
        assert_eq!(&pkt[16..20], &buttons[2].to_wire());
        // 槽位 3 = 后退；界面数组第 5 个是后退
        assert_eq!(&pkt[20..24], &buttons[4].to_wire());
        // 槽位 4 = 前进；界面数组第 4 个是前进
        assert_eq!(&pkt[24..28], &buttons[3].to_wire());
        assert_eq!(&pkt[28..32], &[0x21, 0x55, 0, 0]);
        assert_eq!(&pkt[32..36], &[0x21, 0x38, 1, 0]);
        assert_eq!(&pkt[36..40], &[0x21, 0x38, 0xFF, 0]);
    }

    #[test]
    fn buttons_roundtrip() {
        let buttons = [
            ButtonAction::Left,
            ButtonAction::Middle,
            ButtonAction::Disabled,
            ButtonAction::Backward,
            ButtonAction::Forward,
        ];
        let resp = encode_buttons_response(&buttons);
        let parsed = parse_buttons(&resp).unwrap();
        assert_eq!(parsed, buttons);
    }

    #[test]
    fn unknown_button_action_is_preserved_as_custom() {
        let action = ButtonAction::from_wire([0x21, 0x38, 0x01, 0x00]);
        assert_eq!(action.to_wire(), [0x21, 0x38, 0x01, 0x00]);
        assert!(action.code().starts_with("custom:"));
        assert_eq!(ButtonAction::from_code(&action.code()), Some(action));
    }

    #[test]
    fn battery_and_online_parsers() {
        let mut pkt = [0u8; 64];
        pkt[0] = 0xAA;
        pkt[1] = CMD_BATTERY;
        pkt[8] = 82;
        pkt[9] = 1;
        assert_eq!(parse_battery(&pkt).unwrap(), (82, true));

        pkt[8] = 0;
        assert!(parse_battery(&pkt).is_err());

        let mut online = [0u8; 64];
        online[0] = 0xAA;
        online[1] = CMD_ONLINE;
        online[8] = 1;
        assert!(parse_online(&online).unwrap());
        online[8] = 0;
        assert!(!parse_online(&online).unwrap());
    }

    #[test]
    fn version_parser_trims_padding() {
        let mut pkt = [0u8; 64];
        pkt[0] = 0xAA;
        pkt[1] = CMD_GET_VERSION;
        pkt[8..16].copy_from_slice(b"V1.0.3\0\0");
        pkt[26..32].copy_from_slice(b"250101");
        let (fw, date) = parse_version(&pkt).unwrap();
        assert_eq!(fw, "V1.0.3");
        assert_eq!(date, "250101");
    }

    #[test]
    fn mismatched_response_is_rejected() {
        let mut pkt = [0u8; 64];
        pkt[0] = 0xAA;
        pkt[1] = CMD_GET_BUTTONS;
        let err = parse_config(&pkt).unwrap_err();
        assert_eq!(err.code, crate::ErrorCode::Protocol);
    }

    #[test]
    fn default_buttons_table_matches_factory() {
        assert_eq!(
            DEFAULT_BUTTONS,
            [
                ButtonAction::Left,
                ButtonAction::Right,
                ButtonAction::Middle,
                ButtonAction::Forward,
                ButtonAction::Backward,
            ]
        );
    }
}
