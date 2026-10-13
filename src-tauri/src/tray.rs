use crate::commands::{self, AppState, Snapshot};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Rect};
use tauri_plugin_autostart::ManagerExt;

const MODES: [&str; 5] = ["Auto", "Frio", "Seco", "Ventilar", "Quente"];

/// When focus loss last hid the flyout. Clicking the tray icon blurs the flyout
/// before the click arrives, so a click right after that must not reopen it.
static HIDDEN_AT: Mutex<Option<Instant>> = Mutex::new(None);

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Iniciar com o Windows",
        true,
        autostart_on,
        None::<&str>,
    )?;
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "power", "Ligar/Desligar", true, None::<&str>)?,
            &MenuItem::with_id(app, "scan", "Procurar aparelhos", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &autostart,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?,
        ],
    )?;
    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Ar-condicionado")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, e| match e.id.as_ref() {
            "power" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = commands::toggle_power(&app).await {
                        let _ = app.emit("ac-error", e);
                    }
                });
            }
            "scan" => {
                show_flyout(app, None);
                let _ = app.emit("ac-scan", ());
            }
            "autostart" => {
                let al = app.autolaunch();
                let on = al.is_enabled().unwrap_or(false);
                let _ = if on { al.disable() } else { al.enable() };
                let _ = autostart.set_checked(al.is_enabled().unwrap_or(false));
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = e
            {
                toggle_flyout(tray.app_handle(), rect);
            }
        })
        .build(app)?;
    Ok(())
}

fn toggle_flyout(app: &AppHandle, rect: Rect) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    let just_hidden = HIDDEN_AT
        .lock()
        .unwrap()
        .is_some_and(|t| t.elapsed() < Duration::from_millis(300));
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
    } else if !just_hidden {
        show_flyout(app, Some(rect));
    }
}

/// Shows the flyout above the tray icon (or the bottom-right corner), clamped to the work area.
pub fn show_flyout(app: &AppHandle, anchor: Option<Rect>) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    // Tray rects are physical on Windows, so the scale factor is unused here.
    let center = anchor.map(|r| {
        let p = r.position.to_physical::<f64>(1.0);
        let s = r.size.to_physical::<f64>(1.0);
        (p.x + s.width / 2.0, p.y + s.height / 2.0)
    });
    let monitor = center
        .and_then(|(x, y)| app.monitor_from_point(x, y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let (Some(m), Ok(size), Ok(scale)) = (monitor, w.outer_size(), w.scale_factor()) {
        let wa = m.work_area();
        let margin = (12.0 * m.scale_factor()) as i32;
        // Size as it will be on the target monitor (Windows rescales on a DPI change).
        let size = size
            .to_logical::<f64>(scale)
            .to_physical::<i32>(m.scale_factor());
        let left = wa.position.x + margin;
        let right = wa.position.x + wa.size.width as i32 - size.width - margin;
        let x = center
            .map_or(right, |(cx, _)| cx as i32 - size.width / 2)
            .clamp(left, right.max(left));
        // Taskbar on top (Windows 10): open below it instead.
        let y = match center {
            Some((_, cy)) if cy < wa.position.y as f64 => wa.position.y + margin,
            _ => wa.position.y + wa.size.height as i32 - size.height - margin,
        };
        let _ = w.set_position(PhysicalPosition::new(x, y));
    }
    let _ = w.show();
    let _ = w.set_focus();
    app.state::<AppState>().wake.notify_one();
}

pub fn hide_on_blur(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main")
        && w.is_visible().unwrap_or(false)
    {
        *HIDDEN_AT.lock().unwrap() = Some(Instant::now());
        let _ = w.hide();
    }
}

pub fn set_tooltip_text(app: &AppHandle, text: &str) {
    if let Some(t) = app.tray_by_id("main") {
        let _ = t.set_tooltip(Some(text));
    }
}

pub fn set_tooltip(app: &AppHandle, s: &Snapshot) {
    let text = if s.props.get("Pow") == Some(&1) {
        let temp = s.props.get("SetTem").copied().unwrap_or_default();
        let mode = s
            .props
            .get("Mod")
            .and_then(|m| MODES.get(*m as usize))
            .unwrap_or(&"?");
        format!("{temp}°C · {mode}")
    } else {
        "Desligado".into()
    };
    set_tooltip_text(app, &text);
}
