//! 后台电量监测。
//!
//! 窗口隐藏之后，前端的定时器虽然还在跑，但用户可能从没打开过窗口
//! （开机自启），所以真正可靠的读取放在 Rust 侧：
//!
//! - 优先复用界面已经打开的设备句柄（同一个 `AppState` 锁，避免两路 HID 同时读写）；
//! - 没有句柄时，最多每 5 分钟临时打开一次设备读电量，读完立刻关闭；
//! - 低电量通知只发一次，充电或电量回升 5% 以上后重新武装。

use crate::settings::{MonitorSettings, MonitorStore};
use crate::AppState;
use mhub_core::{Battery, ConnectionKind, DeviceKind, KeyboardDevice, LiveStatus, MouseDevice};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub const EVENT_READING: &str = "monitor-reading";
pub const EVENT_SETTINGS: &str = "monitor-settings";
pub const EVENT_ALERT: &str = "monitor-alert";

/// 没有界面句柄时，临时打开设备的最小间隔。
const PROBE_INTERVAL: Duration = Duration::from_secs(5 * 60);
/// 通知后电量回升多少个百分点才重新武装。
const REARM_MARGIN: u8 = 5;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceReading {
    pub kind: DeviceKind,
    pub key: String,
    pub name: String,
    pub online: bool,
    pub battery: Option<Battery>,
}

impl DeviceReading {
    pub(crate) fn new(
        kind: DeviceKind,
        key: String,
        name: String,
        online: bool,
        battery: Option<Battery>,
    ) -> Self {
        Self {
            kind,
            key,
            name,
            online,
            battery,
        }
    }

    pub(crate) fn label(&self) -> String {
        if !self.online {
            return "已休眠".into();
        }
        match &self.battery {
            Some(battery) if battery.charging => format!("{}%（充电中）", battery.percent),
            Some(battery) => format!("{}%", battery.percent),
            None => "电量未知".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonitorStatus {
    pub mouse: Option<DeviceReading>,
    pub keyboard: Option<DeviceReading>,
    pub read_at: u64,
}

impl MonitorStatus {
    fn contains(&self, key: &str) -> bool {
        self.mouse.as_ref().is_some_and(|r| r.key == key)
            || self.keyboard.as_ref().is_some_and(|r| r.key == key)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryAlert {
    pub kind: DeviceKind,
    pub key: String,
    pub name: String,
    pub percent: u8,
    pub threshold: u8,
    /// 系统通知是否真的显示出来了；失败时前端可以提示一次。
    pub notified: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorConfig {
    pub settings: MonitorSettings,
    pub autostart: bool,
}

pub struct MonitorState {
    pub settings: MonitorStore,
    last: Mutex<MonitorStatus>,
    alerts: Mutex<AlertTracker>,
    refresh: AtomicBool,
    last_probe: Mutex<Option<Instant>>,
}

impl MonitorState {
    pub fn new(settings: MonitorStore) -> Self {
        Self {
            settings,
            last: Mutex::new(MonitorStatus::default()),
            alerts: Mutex::new(AlertTracker::default()),
            refresh: AtomicBool::new(false),
            // 启动后的前 5 分钟不探测：给前端留出自动连接设备的时间。
            last_probe: Mutex::new(Some(Instant::now())),
        }
    }

    pub fn last(&self) -> MonitorStatus {
        self.last
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn set_last(&self, status: MonitorStatus) {
        *self
            .last
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }

    /// 设置变了（开启监测、改间隔、改阈值）之后立刻读一次。
    pub fn request_refresh(&self) {
        self.refresh.store(true, Ordering::SeqCst);
    }

    fn take_refresh(&self) -> bool {
        self.refresh.swap(false, Ordering::SeqCst)
    }

    /// 是否允许这次临时打开设备；顺便记录时间。
    fn take_probe(&self) -> bool {
        let mut last = self
            .last_probe
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let due = last
            .map(|instant| instant.elapsed() >= PROBE_INTERVAL)
            .unwrap_or(true);
        if due {
            *last = Some(Instant::now());
        }
        due
    }
}

pub fn spawn(app: AppHandle) {
    let result = std::thread::Builder::new()
        .name("mhub-battery-monitor".into())
        .spawn(move || run(app));
    if let Err(e) = result {
        eprintln!("[mhub] 启用电量监测线程失败：{e}");
    }
}

fn run(app: AppHandle) {
    // 让主窗口、托盘和前端初始化先完成。
    std::thread::sleep(Duration::from_millis(1200));
    let mut last_read: Option<Instant> = None;

    loop {
        let settings = app.state::<MonitorState>().settings.get();
        if settings.enabled {
            let state = app.state::<MonitorState>();
            let due = state.take_refresh()
                || last_read
                    .map(|at| at.elapsed() >= Duration::from_secs(settings.interval_secs))
                    .unwrap_or(true);
            if due {
                let allow_probe = state.take_probe();
                let status = read_status(&app, allow_probe);
                state.set_last(status.clone());
                let _ = app.emit(EVENT_READING, &status);
                crate::tray::update_battery(&app, &status);
                if settings.notify_low {
                    check_alerts(&app, &settings, &status);
                }
                last_read = Some(Instant::now());
            }
        } else {
            last_read = None;
            let _ = app.state::<MonitorState>().take_refresh();
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn read_status(app: &AppHandle, allow_probe: bool) -> MonitorStatus {
    let (mouse, keyboard) = app.state::<AppState>().read_for_monitor(allow_probe);
    MonitorStatus {
        mouse,
        keyboard,
        read_at: unix_now(),
    }
}

// ---------------------------------------------------------------------------
// 低电量提醒
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct AlertEntry {
    /// 还没提醒过，或者已经充电 / 电量回升。
    armed: bool,
    /// 连续多少轮没有看到这台设备。
    misses: u8,
}

impl Default for AlertEntry {
    fn default() -> Self {
        Self {
            armed: true,
            misses: 0,
        }
    }
}

impl AlertEntry {
    fn evaluate(&mut self, percent: u8, charging: bool, threshold: u8) -> bool {
        if charging || percent > threshold.saturating_add(REARM_MARGIN) {
            self.armed = true;
            return false;
        }
        if percent <= threshold && self.armed {
            self.armed = false;
            return true;
        }
        false
    }
}

#[derive(Debug, Default)]
struct AlertTracker {
    entries: HashMap<String, AlertEntry>,
}

impl AlertTracker {
    fn should_alert(&mut self, reading: &DeviceReading, threshold: u8) -> bool {
        let Some(battery) = reading.battery else {
            // 读不到电量时不改变状态，避免半睡半醒时把提醒状态弄乱。
            return false;
        };
        let entry = self.entries.entry(reading.key.clone()).or_default();
        entry.misses = 0;
        entry.evaluate(battery.percent, battery.charging, threshold)
    }

    /// 设备被拔掉几轮之后清掉它的提醒状态；下次插回来会重新提醒一次。
    fn sweep(&mut self, status: &MonitorStatus) {
        for (key, entry) in self.entries.iter_mut() {
            if !status.contains(key) {
                entry.misses = entry.misses.saturating_add(1);
            }
        }
        self.entries.retain(|_, entry| entry.misses < 3);
    }
}

fn check_alerts(app: &AppHandle, settings: &MonitorSettings, status: &MonitorStatus) {
    let state = app.state::<MonitorState>();
    let mut pending = Vec::new();
    {
        let mut tracker = state
            .alerts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        tracker.sweep(status);
        for reading in [status.mouse.as_ref(), status.keyboard.as_ref()]
            .into_iter()
            .flatten()
        {
            if !reading.online {
                continue;
            }
            if tracker.should_alert(reading, settings.low_threshold) {
                pending.push(reading.clone());
            }
        }
    }

    for reading in pending {
        let Some(battery) = reading.battery else {
            continue;
        };
        let (title, body) = low_battery_copy(&reading, battery.percent, settings.low_threshold);
        let (notified, message) = match app.notification().builder().title(title).body(body).show()
        {
            Ok(()) => (true, None),
            Err(e) => {
                eprintln!("[mhub] 低电量通知发送失败：{e}");
                (false, Some(e.to_string()))
            }
        };
        let alert = BatteryAlert {
            kind: reading.kind,
            key: reading.key.clone(),
            name: reading.name.clone(),
            percent: battery.percent,
            threshold: settings.low_threshold,
            notified,
            message,
        };
        let _ = app.emit(EVENT_ALERT, &alert);
    }
}

fn low_battery_copy(reading: &DeviceReading, percent: u8, threshold: u8) -> (String, String) {
    let title = format!("{} 电量偏低", reading.name);
    let body = if percent <= 5 {
        format!("剩余 {percent}%，请立即充电，设备可能随时断开连接。")
    } else {
        format!("剩余 {percent}%，已低于 {threshold}% 提醒线。接上 USB 线充电即可。")
    };
    (title, body)
}

/// 设置页里的「测试提醒」。
pub fn test_notification(app: &AppHandle) -> Result<(), String> {
    let settings = app.state::<MonitorState>().settings.get();
    let status = app.state::<MonitorState>().last();
    let device = status.mouse.as_ref().or(status.keyboard.as_ref());
    let (title, body) = match device {
        Some(reading) => match reading.battery {
            Some(battery) => (
                format!("{} 电量提醒测试", reading.name),
                format!(
                    "当前电量 {}{}。正式提醒会在电量低于 {}% 时出现。",
                    battery.percent,
                    if battery.charging {
                        "（充电中）"
                    } else {
                        ""
                    },
                    settings.low_threshold
                ),
            ),
            None => (
                format!("{} 电量提醒测试", reading.name),
                format!(
                    "通知可以正常显示。电量低于 {}% 时会收到提醒。",
                    settings.low_threshold
                ),
            ),
        },
        None => (
            "MHUB Lite 电量提醒测试".to_string(),
            format!(
                "通知可以正常显示。连接 MCHOSE 设备后，电量低于 {}% 时会收到提醒。",
                settings.low_threshold
            ),
        ),
    };
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// 没有界面句柄时的兜底读取（只读电量，不读整份配置）
// ---------------------------------------------------------------------------

pub(crate) fn probe_mouse(allow: bool) -> Option<DeviceReading> {
    if !allow {
        return None;
    }
    let list = mhub_core::scan_mice(false).ok()?;
    let summary = list
        .iter()
        .find(|d| d.supported && d.connection == ConnectionKind::Wireless)
        .or_else(|| list.iter().find(|d| d.supported))?;
    let mut device = MouseDevice::open(&summary.key).ok()?;
    let live = device.read_live().unwrap_or(LiveStatus {
        online: false,
        battery: None,
    });
    Some(DeviceReading::new(
        DeviceKind::Mouse,
        summary.key.clone(),
        summary.name.clone(),
        live.online,
        live.battery,
    ))
}

pub(crate) fn probe_keyboard(allow: bool) -> Option<DeviceReading> {
    if !allow {
        return None;
    }
    let list = mhub_core::keyboard::scan().ok()?;
    let summary = list.into_iter().find(|d| d.supported)?;
    let mut device = KeyboardDevice::open(&summary.key).ok()?;
    let live = device.read_live().ok()?;
    Some(DeviceReading::new(
        DeviceKind::Keyboard,
        summary.key,
        summary.name,
        live.online,
        live.battery,
    ))
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

    #[test]
    fn alerts_once_then_rearms_after_charging() {
        let mut entry = AlertEntry::default();
        assert!(!entry.evaluate(80, false, 20));
        assert!(entry.evaluate(20, false, 20));
        assert!(!entry.evaluate(19, false, 20));
        assert!(!entry.evaluate(18, false, 20));
        // 插上线充电后重新武装，再拔线还能提醒一次。
        assert!(!entry.evaluate(15, true, 20));
        assert!(entry.evaluate(15, false, 20));
    }

    #[test]
    fn alerts_rearm_when_battery_recovers() {
        let mut entry = AlertEntry::default();
        assert!(entry.evaluate(12, false, 20));
        assert!(!entry.evaluate(24, false, 20));
        assert!(!entry.evaluate(26, false, 20));
        assert!(entry.evaluate(20, false, 20));
    }

    #[test]
    fn charging_low_battery_does_not_alert() {
        let mut entry = AlertEntry::default();
        assert!(!entry.evaluate(3, true, 20));
        assert!(!entry.evaluate(3, true, 20));
    }

    #[test]
    fn tracker_drops_disconnected_devices() {
        let mut tracker = AlertTracker::default();
        let reading = DeviceReading::new(
            DeviceKind::Mouse,
            "mouse-key".into(),
            "测试鼠标".into(),
            true,
            Some(Battery {
                percent: 10,
                charging: false,
            }),
        );
        assert!(tracker.should_alert(&reading, 20));
        for _ in 0..3 {
            tracker.sweep(&MonitorStatus::default());
        }
        assert!(tracker.entries.is_empty());
    }

    #[test]
    fn monitor_status_serializes_camel_case() {
        let status = MonitorStatus {
            mouse: Some(DeviceReading::new(
                DeviceKind::Mouse,
                "m".into(),
                "鼠标".into(),
                true,
                Some(Battery {
                    percent: 42,
                    charging: false,
                }),
            )),
            ..MonitorStatus::default()
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"readAt\""));
        assert!(json.contains("\"charging\":false"));
    }
}
