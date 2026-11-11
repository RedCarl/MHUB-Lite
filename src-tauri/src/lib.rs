use mhub_core::keyboard::{KeyboardDevice, KeyboardInput, KeyboardSnapshot};
use mhub_core::{
    ButtonAction, DeviceError, DeviceSummary, ErrorCode, LiveStatus, MouseConfig, MouseDevice,
    Snapshot,
};
use serde::Serialize;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{Manager, State};

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
async fn connect(state: State<'_, AppState>, key: String) -> Result<Snapshot, ApiError> {
    if key.starts_with("kb:") {
        return Err(ApiError {
            code: ErrorCode::Invalid,
            message: "这是键盘设备，请使用键盘连接".into(),
        });
    }
    let inner = state.inner.clone();
    blocking(move || {
        {
            let mut guard = lock(&inner)?;
            if let Some(existing) = guard.mouse.as_mut() {
                if existing.info().key == key {
                    return existing.snapshot().map_err(ApiError::from);
                }
            }
            // 释放上一只设备，避免再次打开时被 macOS 的独占访问拦住
            guard.mouse = None;
            guard.saved = None;
        }
        let mut device = MouseDevice::open(&key).map_err(ApiError::from)?;
        let snapshot = device.snapshot();
        let mut guard = lock(&inner)?;
        guard.mouse = Some(device);
        match snapshot {
            Ok(snap) => {
                guard.saved = Some(snap.clone());
                Ok(snap)
            }
            Err(e) => Err(e.into()),
        }
    })
    .await
}

#[tauri::command]
async fn connect_keyboard(
    state: State<'_, AppState>,
    key: String,
) -> Result<KeyboardSnapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        {
            let mut guard = lock(&inner)?;
            if let Some(existing) = guard.keyboard.as_mut() {
                if existing.info.key == key {
                    return Ok(existing.snapshot());
                }
            }
            guard.keyboard = None;
        }
        let device = KeyboardDevice::open(&key).map_err(ApiError::from)?;
        let snapshot = device.snapshot();
        let mut guard = lock(&inner)?;
        guard.keyboard = Some(device);
        Ok(snapshot)
    })
    .await
}

#[tauri::command]
async fn disconnect(state: State<'_, AppState>, kind: Option<String>) -> Result<(), ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        if kind.as_deref() == Some("keyboard") {
            guard.keyboard = None;
        } else {
            guard.mouse = None;
            guard.saved = None;
        }
        Ok(())
    })
    .await
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
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
            app_version
        ])
        .run(tauri::generate_context!())
        .expect("MHUB Lite 启动失败");
}
