//! 托盘 / 菜单栏图标。
//!
//! macOS 上是一个菜单栏图标（模板图，跟随深浅色），Windows 上是任务栏通知区图标。
//! 左键打开主窗口，右键弹出菜单；菜单里的电量文字和两个勾选项会随状态更新。

use crate::monitor::{MonitorState, MonitorStatus};
use crate::settings::MonitorSettings;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const TRAY_ID: &str = "mhub-tray";

const MENU_OPEN: &str = "tray-open";
const MENU_MOUSE: &str = "tray-mouse";
const MENU_KEYBOARD: &str = "tray-keyboard";
const MENU_LOW: &str = "tray-low";
const MENU_AUTOSTART: &str = "tray-autostart";
const MENU_QUIT: &str = "tray-quit";

/// macOS 菜单栏用的白色字形（由 `assets/logo.svg` 生成，模板图会在系统里自动反色）。
const TRAY_ICON: Image<'static> = tauri::include_image!("icons/tray.png");

pub struct TrayHandles {
    mouse: MenuItem<tauri::Wry>,
    keyboard: MenuItem<tauri::Wry>,
    low: CheckMenuItem<tauri::Wry>,
    autostart: CheckMenuItem<tauri::Wry>,
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let settings = app.state::<MonitorState>().settings.get();

    let open = MenuItem::with_id(app, MENU_OPEN, "打开 MHUB Lite", true, None::<&str>)?;
    let mouse = MenuItem::with_id(app, MENU_MOUSE, "鼠标 · 未连接", false, None::<&str>)?;
    let keyboard = MenuItem::with_id(app, MENU_KEYBOARD, "键盘 · 未连接", false, None::<&str>)?;
    let low = CheckMenuItem::with_id(
        app,
        MENU_LOW,
        "低电量提醒",
        true,
        settings.notify_low,
        None::<&str>,
    )?;
    let autostart = CheckMenuItem::with_id(
        app,
        MENU_AUTOSTART,
        "开机自动启动",
        true,
        crate::autostart_enabled(app),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "退出 MHUB Lite", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &open, &sep1, &mouse, &keyboard, &sep2, &low, &autostart, &sep3, &quit,
        ],
    )?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("MHUB Lite · 后台电量监测")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_OPEN => show_main(app),
            MENU_LOW => toggle_low(app),
            MENU_AUTOSTART => toggle_autostart(app),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });

    // Windows 用彩色应用图标；macOS 用单色模板图，避免菜单栏里出现一块黑色方块。
    #[cfg(target_os = "macos")]
    let builder = builder.icon(TRAY_ICON).icon_as_template(true);
    #[cfg(not(target_os = "macos"))]
    let builder = match app.default_window_icon() {
        Some(icon) => builder.icon(icon.clone()),
        None => builder.icon(TRAY_ICON),
    };

    builder.build(app)?;
    app.manage(TrayHandles {
        mouse,
        keyboard,
        low,
        autostart,
    });
    Ok(())
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 设置从任何入口（前端 / 托盘菜单）变化后，同步两个勾选项。
pub fn sync_settings(app: &AppHandle, settings: &MonitorSettings) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return;
    };
    let _ = handles.low.set_checked(settings.notify_low);
    let _ = handles.autostart.set_checked(crate::autostart_enabled(app));
}

/// 每次后台读取后刷新托盘提示和菜单里的电量文字。
pub fn update_battery(app: &AppHandle, status: &MonitorStatus) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return;
    };
    let mouse_text = status
        .mouse
        .as_ref()
        .map(|reading| format!("鼠标 · {}", reading.label()))
        .unwrap_or_else(|| "鼠标 · 未连接".into());
    let keyboard_text = status
        .keyboard
        .as_ref()
        .map(|reading| format!("键盘 · {}", reading.label()))
        .unwrap_or_else(|| "键盘 · 未连接".into());

    let _ = handles.mouse.set_text(&mouse_text);
    let _ = handles.keyboard.set_text(&keyboard_text);

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = format!("MHUB Lite · {mouse_text} · {keyboard_text}");
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn toggle_low(app: &AppHandle) {
    let current = app.state::<MonitorState>().settings.get();
    let mut next = current.clone();
    next.notify_low = !current.notify_low;
    if let Err(e) = crate::update_settings(app, next) {
        eprintln!("[mhub] 切换低电量提醒失败：{e}");
        sync_settings(app, &current);
    }
}

fn toggle_autostart(app: &AppHandle) {
    let current = app.state::<MonitorState>().settings.get();
    let mut next = current.clone();
    next.start_at_login = !crate::autostart_enabled(app);
    if let Err(e) = crate::update_settings(app, next) {
        eprintln!("[mhub] 切换开机启动失败：{e}");
        sync_settings(app, &current);
    }
}
