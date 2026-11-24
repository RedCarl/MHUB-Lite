//! macOS 菜单栏电量面板。
//!
//! 左键点菜单栏图标时不再弹原生菜单，而是在图标下方浮一块小面板：
//! 设备图标 + 大号电量百分比 + 电量条。面板不抢主窗口，点外面 / Esc 收起。

use crate::monitor::MonitorState;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{
    AppHandle, Emitter, LogicalPosition, Manager, Position, WebviewUrl, WebviewWindowBuilder,
};

pub const PANEL_LABEL: &str = "status";

const PANEL_WIDTH: f64 = 320.0;
const PANEL_HEIGHT: f64 = 300.0;

/// 点面板外面时延迟收起；如果紧接着点了菜单栏图标，用这个代数取消那次收起。
static HIDE_GENERATION: AtomicU64 = AtomicU64::new(0);
/// 面板最近一次拿到焦点的时间（毫秒），用来忽略激活过程中的焦点抖动。
static FOCUSED_AT: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(PANEL_LABEL).is_some() {
        return Ok(());
    }

    let panel = WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("index.html".into()))
        .title("MHUB Lite 电量")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .accept_first_mouse(true)
        .visible(false)
        .build()?;

    // 在别的 Space / 全屏应用上也能弹出来。
    let _ = panel.set_visible_on_all_workspaces(true);

    Ok(())
}

/// 左键点菜单栏图标：面板开着就收起，否则定位到图标下方再弹出来。
pub fn toggle(app: &AppHandle, rect: tauri::Rect) {
    let Some(panel) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };

    // 取消可能还在排队的「失焦收起」。
    HIDE_GENERATION.fetch_add(1, Ordering::SeqCst);

    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }

    position(app, &panel, rect);
    let monitor = app.state::<MonitorState>();
    monitor.request_probe();
    monitor.request_refresh();
    FOCUSED_AT.store(now_ms(), Ordering::SeqCst);
    let _ = panel.show();
    let _ = panel.set_focus();
    let _ = app.emit("panel-shown", ());
}

pub fn hide(app: &AppHandle) {
    HIDE_GENERATION.fetch_add(1, Ordering::SeqCst);
    if let Some(panel) = app.get_webview_window(PANEL_LABEL) {
        let _ = panel.hide();
    }
}

/// 面板重新拿到焦点时，取消还在排队的「失焦收起」。
///
/// 从后台激活 App 的过程中会先收到一次 Focused(false) 再收到 Focused(true)，
/// 不取消的话面板会刚出现就自己收起来。
pub fn keep_open() {
    FOCUSED_AT.store(now_ms(), Ordering::SeqCst);
    HIDE_GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// 失焦后延迟收起：给「点图标收起」留一点时间，避免刚收起又被重新打开。
pub fn hide_soon(app: &AppHandle) {
    // 激活 App 时会先 false 再 true；刚弹出 600ms 内的失焦不算「点外面」。
    if now_ms().saturating_sub(FOCUSED_AT.load(Ordering::SeqCst)) < 600 {
        return;
    }
    let app = app.clone();
    let generation = HIDE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(160));
        if HIDE_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        let Some(panel) = app.get_webview_window(PANEL_LABEL) else {
            return;
        };
        if !panel.is_focused().unwrap_or(false) {
            let _ = panel.hide();
        }
    });
}

/// 面板贴在菜单栏图标正下方，并夹在当前屏幕的工作区内。
fn position(app: &AppHandle, panel: &tauri::WebviewWindow, rect: tauri::Rect) {
    // 实测 macOS 上 tray 事件里的位置/尺寸是屏幕坐标（pt），虽然类型写的是 Physical。
    // 直接当逻辑坐标用，和 NSWindow 的 frame 对得上。
    let icon = rect.position.to_physical::<f64>(1.0);
    let icon_size = rect.size.to_physical::<f64>(1.0);
    let icon_x = icon.x;
    let icon_y = icon.y;
    let icon_w = icon_size.width;
    let icon_h = icon_size.height;

    let mut x = icon_x + icon_w / 2.0 - PANEL_WIDTH / 2.0;
    let mut y = icon_y + icon_h;

    // monitor_from_point 收的是逻辑坐标（CGDisplayBounds），不是物理像素。
    if let Ok(Some(monitor)) = app.monitor_from_point(icon_x, icon_y) {
        let mp = monitor.position();
        let ms = monitor.size();
        let scale = monitor.scale_factor();
        let logical_w = ms.width as f64 / scale;
        let logical_h = ms.height as f64 / scale;
        let left = mp.x as f64 + 8.0;
        let top = mp.y as f64 + 8.0;
        let right = (mp.x as f64 + logical_w - PANEL_WIDTH - 8.0).max(left);
        let bottom = (mp.y as f64 + logical_h - PANEL_HEIGHT - 8.0).max(top);
        x = x.clamp(left, right);
        y = y.clamp(top, bottom);
    }

    let _ = panel.set_position(Position::Logical(LogicalPosition::new(x, y)));
}
