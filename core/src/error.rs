use serde::{Deserialize, Serialize};
use std::fmt;

/// 前端按 code 决定用哪句中文，message 只作为补充/排错信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    /// 没有找到设备
    NotFound,
    /// 系统拒绝访问 HID（Linux udev / macOS 权限）
    Permission,
    /// 无线接收器在，但鼠标休眠 / 关机
    Offline,
    /// 设备没有在超时内回应
    Timeout,
    /// 回应了，但内容不符合协议
    Protocol,
    /// 型号存在，但当前版本还没有验证过的协议
    Unsupported,
    /// HID 子系统报错
    Hid,
    /// 设备正被别的操作占用
    Busy,
    /// 参数不合法（写入前校验）
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceError {
    pub code: ErrorCode,
    pub message: String,
}

impl DeviceError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn permission(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Permission, message)
    }

    pub fn offline(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Offline, message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }

    pub fn protocol(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Protocol, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unsupported, message)
    }

    pub fn hid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Hid, message)
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Invalid, message)
    }

    pub fn busy(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Busy, message)
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for DeviceError {}

impl From<hidapi::HidError> for DeviceError {
    fn from(e: hidapi::HidError) -> Self {
        let text = e.to_string();
        let lower = text.to_lowercase();
        // macOS 上「没有权限」和「设备没插」是两种完全不同的处理方式，
        // 不要都吞成一句「打开失败」。
        if lower.contains("exclusive") || lower.contains("already open") {
            return DeviceError::busy(
                "设备被其他程序独占：请关闭 MCHOSE HUB、浏览器里的 MCHOSE 网页驱动或其他占用 HID 的程序后重试",
            );
        }
        if lower.contains("not permitted") {
            return DeviceError::permission(
                "macOS 拒绝访问设备 HID：请在「系统设置 → 隐私与安全性 → 输入监控」中允许 MHUB Lite，然后重启应用",
            );
        }
        if lower.contains("permission")
            || lower.contains("access")
            || lower.contains("not permitted")
            || lower.contains("privilege")
        {
            return DeviceError::permission(text);
        }
        DeviceError::hid(text)
    }
}
