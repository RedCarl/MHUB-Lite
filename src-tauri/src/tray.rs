//! 托盘 / 菜单栏图标。
//!
//! macOS 上是一个菜单栏图标（模板图，跟随深浅色）：左键弹出电量面板，
//! 右键是设置菜单；Windows 上是任务栏通知区图标：左键打开主窗口、右键弹菜单。

use crate::monitor::{DeviceReading, MonitorState, MonitorStatus};
use crate::settings::MonitorSettings;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const TRAY_ID: &str = "mhub-tray";

const MENU_LOW: &str = "tray-low";
const MENU_AUTOSTART: &str = "tray-autostart";
const MENU_QUIT: &str = "tray-quit";

/// macOS 菜单栏用的白色字形（由 assets/logo.svg 生成，模板图会在系统里自动反色）。
const TRAY_ICON: Image<'static> = tauri::include_image!("icons/tray.png");

pub struct TrayHandles {
    low: CheckMenuItem<tauri::Wry>,
    autostart: CheckMenuItem<tauri::Wry>,
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let settings = app.state::<MonitorState>().settings.get();

    // 右键菜单只留常驻相关的开关，电量交给面板。
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
    let sep = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&low, &autostart, &sep, &quit])?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("MHUB Lite · 后台电量监测")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_LOW => toggle_low(app),
            MENU_AUTOSTART => toggle_autostart(app),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                #[cfg(target_os = "macos")]
                {
                    crate::panel::toggle(tray.app_handle(), rect);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = rect;
                    show_main(tray.app_handle());
                }
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
    app.manage(TrayHandles { low, autostart });
    Ok(())
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 设置从任何入口（前端 / 托盘菜单）变化后，同步两个勾选项和托盘提示。
pub fn sync_settings(app: &AppHandle, settings: &MonitorSettings) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return;
    };
    let _ = handles.low.set_checked(settings.notify_low);
    let _ = handles.autostart.set_checked(crate::autostart_enabled(app));

    if settings.enabled {
        update_battery(app, &app.state::<MonitorState>().last());
    } else if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some("MHUB Lite · 电量监测已关闭"));
    }
}

/// 每次后台读取后刷新托盘提示；面板自己订阅 monitor-reading，不依赖这里。
pub fn update_battery(app: &AppHandle, status: &MonitorStatus) {
    let threshold = app.state::<MonitorState>().settings.get().low_threshold;
    let mouse_text = status_line("鼠标", status.mouse.as_ref(), threshold);
    let keyboard_text = status_line("键盘", status.keyboard.as_ref(), threshold);

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = format!("MHUB Lite · {mouse_text} · {keyboard_text}");
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

/// 菜单栏提示里的一行设备状态，例如「鼠标 · 77%（充电中）」。
///
/// 读不到电量、设备睡着了、或者根本没连，都要说清楚是哪一种，
/// 不能都显示成「未连接」。
fn status_line(label: &str, reading: Option<&DeviceReading>, threshold: u8) -> String {
    let Some(reading) = reading else {
        return format!("{label} · 未连接");
    };
    if !reading.online {
        return format!("{label} · 已休眠");
    }
    match &reading.battery {
        Some(battery) if battery.charging => format!("{label} · {}%（充电中）", battery.percent),
        Some(battery) if battery.percent <= threshold => {
            format!("{label} · {}%（电量偏低）", battery.percent)
        }
        Some(battery) => format!("{label} · {}%", battery.percent),
        None => format!("{label} · 电量未知"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use mhub_core::{Battery, DeviceKind};

    fn reading(online: bool, battery: Option<Battery>) -> DeviceReading {
        DeviceReading::new(
            DeviceKind::Mouse,
            "test-key".into(),
            "MCHOSE A7 V3 Ultra+".into(),
            online,
            battery,
        )
    }

    #[test]
    fn status_line_names_every_state() {
        assert_eq!(status_line("鼠标", None, 20), "鼠标 · 未连接");
        assert_eq!(
            status_line("鼠标", Some(&reading(false, None)), 20),
            "鼠标 · 已休眠"
        );
        assert_eq!(
            status_line(
                "鼠标",
                Some(&reading(
                    true,
                    Some(Battery {
                        percent: 8,
                        charging: true
                    })
                )),
                20
            ),
            "鼠标 · 8%（充电中）"
        );
        assert_eq!(
            status_line(
                "鼠标",
                Some(&reading(
                    true,
                    Some(Battery {
                        percent: 18,
                        charging: false
                    })
                )),
                20
            ),
            "鼠标 · 18%（电量偏低）"
        );
        assert_eq!(
            status_line(
                "鼠标",
                Some(&reading(
                    true,
                    Some(Battery {
                        percent: 77,
                        charging: false
                    })
                )),
                20
            ),
            "鼠标 · 77%"
        );
        assert_eq!(
            status_line("键盘", Some(&reading(true, None)), 20),
            "键盘 · 电量未知"
        );
    }
}
