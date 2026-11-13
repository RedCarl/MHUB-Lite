mod monitor;
mod settings;
mod tray;

use mhub_core::keyboard::{KeyboardDevice, KeyboardInput, KeyboardSnapshot};
use mhub_core::{
    ButtonAction, DeviceError, DeviceKind, DeviceSummary, ErrorCode, LiveStatus, MouseConfig,
    MouseDevice, Snapshot,
};
use monitor::{DeviceReading, MonitorConfig, MonitorState, MonitorStatus};
use serde::Serialize;
use settings::{MonitorSettings, MonitorStore};
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl From<DeviceError> for ApiError {
    fn from(e: DeviceError) -> Self {
        Self {
            code: e.code,
            message: e.message,
        }
    }
}

fn busy(message: impl Into<String>) -> ApiError {
    ApiError {
        code: ErrorCode::Busy,
        message: message.into(),
    }
}

fn not_connected(what: &str) -> ApiError {
    ApiError {
        code: ErrorCode::Offline,
        message: format!("还没有连接{what}"),
    }
}

fn system_error(message: impl Into<String>) -> ApiError {
    ApiError {
        code: ErrorCode::Hid,
        message: message.into(),
    }
}

struct Inner {
    mouse: Option<MouseDevice>,
    keyboard: Option<KeyboardDevice>,
    saved: Option<Snapshot>,
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                mouse: None,
                keyboard: None,
                saved: None,
            })),
        }
    }

    /// 后台电量监测用：优先复用界面已打开的设备，避免两路 HID 同时读写。
    ///
    /// 整个读取过程都握着同一把锁；没有句柄时（例如开机自启后前端还没连接），
    /// 最多每 5 分钟临时打开设备读一次电量。这里只读状态，不写配置。
    pub fn read_for_monitor(
        &self,
        allow_probe: bool,
    ) -> (Option<DeviceReading>, Option<DeviceReading>) {
        let mut mouse = None;
        let mut keyboard = None;
        let mut has_mouse = false;
        let mut has_keyboard = false;

        let mut guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => return (None, None),
        };

        if let Some(device) = guard.mouse.as_mut() {
            has_mouse = true;
            let info = device.info().clone();
            let live = device.read_live().unwrap_or(LiveStatus {
                online: false,
                battery: None,
            });
            mouse = Some(DeviceReading::new(
                DeviceKind::Mouse,
                info.key,
                info.product_name,
                live.online,
                live.battery,
            ));
        }

        if let Some(device) = guard.keyboard.as_mut() {
            has_keyboard = true;
            let info = device.info.clone();
            let live = device.read_live().unwrap_or(LiveStatus {
                online: false,
                battery: None,
            });
            keyboard = Some(DeviceReading::new(
                DeviceKind::Keyboard,
                info.key,
                info.product_name,
                live.online,
                live.battery,
            ));
        }

        if !has_mouse {
            mouse = monitor::probe_mouse(allow_probe);
        }
        if !has_keyboard {
            keyboard = monitor::probe_keyboard(allow_probe);
        }

        (mouse, keyboard)
    }
}

fn lock(inner: &Arc<Mutex<Inner>>) -> Result<MutexGuard<'_, Inner>, ApiError> {
    inner
        .lock()
        .map_err(|_| busy("设备状态锁已失效，请重启应用"))
}

async fn blocking<T, F>(f: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ApiError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| busy(format!("后台任务没有完成：{e}")))?
}

#[tauri::command]
async fn scan_devices(
    state: State<'_, AppState>,
    include_unknown: Option<bool>,
) -> Result<Vec<DeviceSummary>, ApiError> {
    let _ = state;
    blocking(move || {
        let mut devices =
            mhub_core::scan_mice(include_unknown.unwrap_or(true)).map_err(ApiError::from)?;
        // 键盘扫描失败（例如权限不足）不应该让鼠标列表整页消失。
        if let Ok(mut keyboards) = mhub_core::keyboard::scan() {
            devices.append(&mut keyboards);
        }
        devices.sort_by_key(|d| (d.kind as u8, !d.supported));
        Ok(devices)
    })
    .await
}

#[tauri::command]
async fn connect(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
) -> Result<Snapshot, ApiError> {
    if key.starts_with("kb:") {
        return Err(ApiError {
            code: ErrorCode::Invalid,
            message: "这是键盘设备，请使用键盘连接".into(),
        });
    }
    let inner = state.inner.clone();
    let result = blocking(move || {
        // 打开设备的过程也握着锁：后台监测线程可能同时在读电量，
        // 两个句柄同时访问同一台设备会打断协议。
        let mut guard = lock(&inner)?;
        if let Some(existing) = guard.mouse.as_mut() {
            if existing.info().key == key {
                return existing.snapshot().map_err(ApiError::from);
            }
        }
        // 释放上一只设备，避免再次打开时被 macOS 的独占访问拦住
        guard.mouse = None;
        guard.saved = None;
        let mut device = MouseDevice::open(&key).map_err(ApiError::from)?;
        let snapshot = device.snapshot();
        guard.mouse = Some(device);
        match snapshot {
            Ok(snap) => {
                guard.saved = Some(snap.clone());
                Ok(snap)
            }
            Err(e) => Err(e.into()),
        }
    })
    .await;
    // 让后台监测马上读到刚连接上的设备，而不是等下个检查间隔。
    if result.is_ok() {
        app.state::<MonitorState>().request_refresh();
    }
    result
}

#[tauri::command]
async fn connect_keyboard(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
) -> Result<KeyboardSnapshot, ApiError> {
    let inner = state.inner.clone();
    let result = blocking(move || {
        // 键盘打开时要读整份配置，同样放在锁里，避免和后台电量读取交错。
        let mut guard = lock(&inner)?;
        if let Some(existing) = guard.keyboard.as_mut() {
            if existing.info.key == key {
                return Ok(existing.snapshot());
            }
        }
        guard.keyboard = None;
        let device = KeyboardDevice::open(&key).map_err(ApiError::from)?;
        let snapshot = device.snapshot();
        guard.keyboard = Some(device);
        Ok(snapshot)
    })
    .await;
    if result.is_ok() {
        app.state::<MonitorState>().request_refresh();
    }
    result
}

#[tauri::command]
async fn disconnect(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: Option<String>,
) -> Result<(), ApiError> {
    let inner = state.inner.clone();
    let result = blocking(move || {
        let mut guard = lock(&inner)?;
        if kind.as_deref() == Some("keyboard") {
            guard.keyboard = None;
        } else {
            guard.mouse = None;
            guard.saved = None;
        }
        Ok(())
    })
    .await;
    if result.is_ok() {
        app.state::<MonitorState>().request_refresh();
    }
    result
}

#[tauri::command]
async fn refresh(state: State<'_, AppState>) -> Result<Snapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let saved = guard.saved.clone();
        let snap = {
            let device = guard.mouse.as_mut().ok_or_else(|| not_connected("鼠标"))?;
            device.snapshot().map_err(ApiError::from)?
        };
        let _ = saved;
        guard.saved = Some(snap.clone());
        Ok(snap)
    })
    .await
}

#[tauri::command]
async fn refresh_keyboard(state: State<'_, AppState>) -> Result<KeyboardSnapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let device = guard
            .keyboard
            .as_mut()
            .ok_or_else(|| not_connected("键盘"))?;
        device.refresh().map_err(ApiError::from)
    })
    .await
}

#[tauri::command]
async fn live_status(state: State<'_, AppState>) -> Result<LiveStatus, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let device = guard.mouse.as_mut().ok_or_else(|| not_connected("鼠标"))?;
        device.read_live().map_err(ApiError::from)
    })
    .await
}

#[tauri::command]
async fn keyboard_live(state: State<'_, AppState>) -> Result<LiveStatus, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let device = guard
            .keyboard
            .as_mut()
            .ok_or_else(|| not_connected("键盘"))?;
        device.read_live().map_err(ApiError::from)
    })
    .await
}

#[tauri::command]
async fn apply_config(
    state: State<'_, AppState>,
    config: MouseConfig,
    buttons: Vec<ButtonAction>,
) -> Result<Snapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        if !buttons.is_empty() && buttons.len() != 5 {
            return Err(ApiError {
                code: ErrorCode::Invalid,
                message: "按键映射必须正好 5 项".into(),
            });
        }
        let mut guard = lock(&inner)?;
        let saved = guard.saved.clone();
        let snap = {
            let device = guard.mouse.as_mut().ok_or_else(|| not_connected("鼠标"))?;
            device
                .apply(&config, &buttons, saved.as_ref())
                .map_err(ApiError::from)?
        };
        guard.saved = Some(snap.clone());
        Ok(snap)
    })
    .await
}

#[tauri::command]
async fn apply_keyboard(
    state: State<'_, AppState>,
    input: KeyboardInput,
) -> Result<KeyboardSnapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let device = guard
            .keyboard
            .as_mut()
            .ok_or_else(|| not_connected("键盘"))?;
        device.apply(&input).map_err(ApiError::from)
    })
    .await
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ---------------------------------------------------------------------------
// 后台常驻与电量提醒
// ---------------------------------------------------------------------------

fn autostart_enabled(app: &AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

fn set_autostart(app: &AppHandle, on: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if on {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|e| format!("设置开机启动失败：{e}"))
}

/// 所有设置入口（前端、托盘菜单）都走这里：先落盘，再同步托盘和界面。
fn update_settings(app: &AppHandle, next: MonitorSettings) -> Result<MonitorConfig, String> {
    let next = next.sanitized();
    if next.start_at_login != autostart_enabled(app) {
        set_autostart(app, next.start_at_login)?;
    }
    let saved = app.state::<MonitorState>().settings.save(next);
    let config = MonitorConfig {
        settings: saved,
        autostart: autostart_enabled(app),
    };
    tray::sync_settings(app, &config.settings);
    app.state::<MonitorState>().request_refresh();
    let _ = app.emit(monitor::EVENT_SETTINGS, &config);
    Ok(config)
}

#[tauri::command]
async fn monitor_config(app: AppHandle) -> MonitorConfig {
    let settings = app.state::<MonitorState>().settings.get();
    MonitorConfig {
        settings,
        autostart: autostart_enabled(&app),
    }
}

#[tauri::command]
fn monitor_status(app: AppHandle) -> MonitorStatus {
    app.state::<MonitorState>().last()
}

#[tauri::command]
async fn set_monitor_settings(
    app: AppHandle,
    settings: MonitorSettings,
) -> Result<MonitorConfig, ApiError> {
    update_settings(&app, settings).map_err(system_error)
}

#[tauri::command]
async fn test_battery_notification(app: AppHandle) -> Result<(), ApiError> {
    monitor::test_notification(&app).map_err(system_error)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

fn launched_minimized() -> bool {
    std::env::args().any(|arg| arg == "--minimized")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--minimized")
                .app_name("MHUB Lite")
                .build(),
        )
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            scan_devices,
            connect,
            connect_keyboard,
            disconnect,
            refresh,
            refresh_keyboard,
            live_status,
            keyboard_live,
            apply_config,
            apply_keyboard,
            app_version,
            monitor_config,
            monitor_status,
            set_monitor_settings,
            test_battery_notification,
            quit_app
        ])
        .setup(|app| {
            let handle = app.handle();
            let store = MonitorStore::load(handle);
            let settings = store.get();
            app.manage(MonitorState::new(store));
            tray::setup(handle)?;
            tray::sync_settings(handle, &settings);
            // 应用可能被移动过，开机自启的路径需要跟着更新。
            if settings.start_at_login {
                let _ = set_autostart(handle, true);
            }
            // 窗口在配置里是隐藏创建的：正常启动时在这里显示，开机自启
            // （--minimized）则保持隐藏，避免登录时闪一下窗口。
            if let Some(window) = app.get_webview_window("main") {
                if launched_minimized() {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            monitor::spawn(handle.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                if app.state::<MonitorState>().settings.get().close_to_tray {
                    // 关闭窗口 = 藏到托盘，电量监测继续。
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    // 用户明确不要后台常驻：关窗即退出，不留一个没有窗口的进程。
                    app.exit(0);
                }
            }
        });

    let app = builder
        .build(tauri::generate_context!())
        .expect("MHUB Lite 启动失败");

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            // 窗口藏起来之后点 Dock 图标，要能重新打开。
            tray::show_main(app_handle);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app_handle, event);
        }
    });
}
