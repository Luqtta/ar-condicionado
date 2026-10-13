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
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            tray::show_flyout(app, None)
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            app.manage(commands::AppState::new(config::load(app.handle())));
            tray::setup(app.handle())?;
            commands::spawn_poller(app.handle().clone());
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
            commands::send,
            commands::rename,
            commands::show,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
