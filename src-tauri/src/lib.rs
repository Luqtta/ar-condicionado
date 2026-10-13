mod commands;
mod config;
pub mod gree;
mod tray;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // must be the first plugin
        .plugin(tauri_plugin_single_instance::init(|app, _, _| tray::show_flyout(app, None)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let cfg = config::load(app.handle());
            let first_run = cfg.device.is_none();
            app.manage(commands::AppState::new(cfg));
            tray::setup(app.handle())?;
            commands::spawn_poller(app.handle().clone());
            if first_run {
                tray::show_flyout(app.handle(), None);
            }
            Ok(())
        })
        .on_window_event(|w, e| match e {
            WindowEvent::Focused(false) => tray::hide_on_blur(w.app_handle()),
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = w.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_info,
            commands::get_snapshot,
            commands::scan,
            commands::bind,
            commands::refresh,
            commands::send,
            commands::rename,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
