//! 后台常驻与电量提醒的设置。
//!
//! 这些值要能在窗口关掉之后继续生效，所以不放在前端的 localStorage，
//! 而是写在应用配置目录的 `monitor.json` 里。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub const SETTINGS_FILE: &str = "monitor.json";

/// 电量提醒阈值允许的范围（百分比）。
pub const MIN_THRESHOLD: u8 = 5;
pub const MAX_THRESHOLD: u8 = 50;
/// 后台读取电量的间隔范围（秒）。
pub const MIN_INTERVAL: u64 = 15;
pub const MAX_INTERVAL: u64 = 1800;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MonitorSettings {
    /// 关闭窗口时隐藏到托盘 / 菜单栏，而不是退出应用。
    pub close_to_tray: bool,
    /// 后台电量监测（窗口隐藏后继续）。
    pub enabled: bool,
    /// 低于阈值时发系统通知。
    pub notify_low: bool,
    /// 电量低于等于这个百分比时提醒。
    pub low_threshold: u8,
    /// 两次后台读取之间的间隔。
    pub interval_secs: u64,
    /// 开机自动启动（写登录项 / 注册表 Run 键）。
    pub start_at_login: bool,
}

impl Default for MonitorSettings {
    fn default() -> Self {
        Self {
            close_to_tray: true,
            enabled: true,
            notify_low: true,
            low_threshold: 20,
            interval_secs: 60,
            start_at_login: false,
        }
    }
}

impl MonitorSettings {
    /// 前端和配置文件都不可信：写入和读取时都收一次边界。
    pub fn sanitized(mut self) -> Self {
        self.low_threshold = self.low_threshold.clamp(MIN_THRESHOLD, MAX_THRESHOLD);
        self.interval_secs = self.interval_secs.clamp(MIN_INTERVAL, MAX_INTERVAL);
        self
    }
}

/// 读写 `monitor.json`。读取失败时回落到默认值，不阻塞应用启动。
pub struct MonitorStore {
    path: Option<PathBuf>,
    settings: Mutex<MonitorSettings>,
}

impl MonitorStore {
    pub fn load(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .ok()
            .map(|dir| dir.join(SETTINGS_FILE));
        let mut settings = MonitorSettings::default();
        if let Some(file) = &path {
            if let Ok(raw) = fs::read_to_string(file) {
                match serde_json::from_str::<MonitorSettings>(&raw) {
                    Ok(parsed) => settings = parsed.sanitized(),
                    Err(e) => eprintln!("[mhub] 读取 {} 失败：{e}", file.display()),
                }
            }
        }
        Self {
            path,
            settings: Mutex::new(settings),
        }
    }

    pub fn get(&self) -> MonitorSettings {
        self.settings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// 保存并返回实际写入的值（已经过 `sanitized`）。
    pub fn save(&self, next: MonitorSettings) -> MonitorSettings {
        let next = next.sanitized();
        {
            let mut guard = self
                .settings
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = next.clone();
        }
        self.write(&next);
        next
    }

    fn write(&self, value: &MonitorSettings) {
        let Some(path) = &self.path else { return };
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("[mhub] 创建配置目录 {} 失败：{e}", parent.display());
                return;
            }
        }
        match serde_json::to_vec_pretty(value) {
            Ok(bytes) => {
                if let Err(e) = fs::write(path, bytes) {
                    eprintln!("[mhub] 写入 {} 失败：{e}", path.display());
                }
            }
            Err(e) => eprintln!("[mhub] 序列化设置失败：{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_and_interval_are_clamped() {
        let s = MonitorSettings {
            low_threshold: 0,
            interval_secs: 3,
            ..MonitorSettings::default()
        }
        .sanitized();
        assert_eq!(s.low_threshold, MIN_THRESHOLD);
        assert_eq!(s.interval_secs, MIN_INTERVAL);

        let s = MonitorSettings {
            low_threshold: 99,
            interval_secs: 99_999,
            ..MonitorSettings::default()
        }
        .sanitized();
        assert_eq!(s.low_threshold, MAX_THRESHOLD);
        assert_eq!(s.interval_secs, MAX_INTERVAL);
    }

    #[test]
    fn old_config_files_keep_working() {
        // 只写了旧字段的文件，其余字段应该回落到默认值。
        let parsed: MonitorSettings = serde_json::from_str(r#"{"lowThreshold":25}"#).unwrap();
        assert_eq!(parsed.low_threshold, 25);
        assert!(parsed.enabled);
        assert!(parsed.close_to_tray);
        assert_eq!(parsed.interval_secs, 60);
    }

    #[test]
    fn settings_serialize_in_camel_case() {
        let json = serde_json::to_string(&MonitorSettings::default()).unwrap();
        assert!(json.contains("\"closeToTray\""));
        assert!(json.contains("\"lowThreshold\""));
        assert!(json.contains("\"startAtLogin\""));
    }
}
