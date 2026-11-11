use mhub_core::{
    demo_summary, scan_devices as scan_hid_devices, ButtonAction, Device, DeviceError,
    DeviceSummary, ErrorCode, LiveStatus, MouseConfig, Snapshot,
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

fn not_connected() -> ApiError {
    ApiError {
        code: ErrorCode::Offline,
        message: "还没有连接设备".into(),
    }
}

struct Inner {
    device: Option<Device>,
    saved: Option<Snapshot>,
    demo: bool,
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

impl AppState {
    fn new(demo: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                device: None,
                saved: None,
                demo,
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
    let inner = state.inner.clone();
    blocking(move || {
        let demo = lock(&inner)?.demo;
        if demo {
            return Ok(vec![demo_summary()]);
        }
        let mut devices =
            scan_hid_devices(include_unknown.unwrap_or(true)).map_err(ApiError::from)?;
        devices.shrink_to_fit();
        Ok(devices)
    })
    .await
}

#[tauri::command]
async fn connect(state: State<'_, AppState>, key: String) -> Result<Snapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let demo = lock(&inner)?.demo;
        let mut device = Device::open(&key, demo).map_err(ApiError::from)?;
        let snapshot = device.snapshot();

        let mut guard = lock(&inner)?;
        guard.device = Some(device);
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
async fn disconnect(state: State<'_, AppState>) -> Result<(), ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        guard.device = None;
        guard.saved = None;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn refresh(state: State<'_, AppState>) -> Result<Snapshot, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let device = guard.device.as_mut().ok_or_else(not_connected)?;
        let snap = device.snapshot().map_err(ApiError::from)?;
        guard.saved = Some(snap.clone());
        Ok(snap)
    })
    .await
}

#[tauri::command]
async fn live_status(state: State<'_, AppState>) -> Result<LiveStatus, ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        let device = guard.device.as_mut().ok_or_else(not_connected)?;
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
        if buttons.len() != 5 {
            return Err(ApiError {
                code: ErrorCode::Invalid,
                message: "按键映射必须正好 5 项".into(),
            });
        }
        let mut arr = [ButtonAction::Left; 5];
        arr.copy_from_slice(&buttons);

        let mut guard = lock(&inner)?;

        // 比较对象是「上一次从设备读回来的样子」，
        // 这样只改 DPI 时不会顺手把按键区重写一遍（写按键会让 HID 端点短暂重枚举）。
        let saved = guard.saved.clone();
        let config_changed = saved
            .as_ref()
            .and_then(|s| s.config.as_ref())
            .map(|c| c != &config)
            .unwrap_or(true);
        let buttons_changed = saved
            .as_ref()
            .and_then(|s| s.buttons.as_ref())
            .map(|b| b != &buttons)
            .unwrap_or(true);

        let snap = {
            let device = guard.device.as_mut().ok_or_else(not_connected)?;
            if config_changed {
                device.set_config(&config).map_err(ApiError::from)?;
            }
            if buttons_changed {
                device.set_buttons(&arr).map_err(ApiError::from)?;
            }
            device.snapshot().map_err(ApiError::from)?
        };

        guard.saved = Some(snap.clone());
        Ok(snap)
    })
    .await
}

#[tauri::command]
async fn set_demo_mode(state: State<'_, AppState>, enabled: bool) -> Result<(), ApiError> {
    let inner = state.inner.clone();
    blocking(move || {
        let mut guard = lock(&inner)?;
        guard.demo = enabled;
        guard.device = None;
        guard.saved = None;
        Ok(())
    })
    .await
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let demo = std::env::var("MHUB_DEMO")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    tauri::Builder::default()
        // 只允许一个实例：两个窗口同时写同一只鼠标的板载 Flash 不是好主意。
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(AppState::new(demo))
        .invoke_handler(tauri::generate_handler![
            scan_devices,
            connect,
            disconnect,
            refresh,
            live_status,
            apply_config,
            set_demo_mode,
            app_version
        ])
        .run(tauri::generate_context!())
        .expect("MHUB Lite 启动失败");
}
