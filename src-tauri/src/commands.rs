use crate::config::{self, Config};
use crate::gree::device::{self, Device, STATUS_COLS};
use crate::gree::discovery::{self, Found};
use crate::tray;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Notify;

const SCAN_WAIT: Duration = Duration::from_secs(3);

pub struct AppState {
    pub cfg: Mutex<Config>,
    /// Serializes all traffic to the unit so a poll and a command never interleave.
    io: tokio::sync::Mutex<()>,
    last: Mutex<Option<Snapshot>>,
    /// greeclimate's "TemSen has no +40 offset" flag, decided on the first reading.
    v4: Mutex<Option<bool>>,
    /// Wakes the poller early (flyout opened, device bound).
    pub wake: Notify,
}

impl AppState {
    pub fn new(cfg: Config) -> Self {
        Self { cfg: Mutex::new(cfg), io: Default::default(), last: Default::default(), v4: Default::default(), wake: Notify::new() }
    }
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub props: BTreeMap<String, i64>,
    pub room: Option<i64>,
}

#[derive(Clone, Serialize)]
pub struct Info {
    name: String,
    ip: String,
    mac: String,
}

fn info(cfg: &Config) -> Option<Info> {
    cfg.device.as_ref().map(|d| Info { name: cfg.name.clone(), ip: d.ip.to_string(), mac: d.mac.clone() })
}

/// Runs `op` against the saved device. On timeout, rescans by MAC in case DHCP
/// moved it, saves the new IP and retries once.
async fn with_device<T, F, Fut>(app: &AppHandle, op: F) -> Result<T, String>
where
    F: Fn(Device) -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let state = app.state::<AppState>();
    let _io = state.io.lock().await;
    let dev = state.cfg.lock().unwrap().device.clone().ok_or("nenhum aparelho configurado")?;
    match op(dev.clone()).await {
        Err(e) if device::is_timeout(&e) => {
            let moved = discovery::scan(SCAN_WAIT).await?.into_iter().find(|f| f.mac == dev.mac && f.ip != dev.ip);
            let Some(found) = moved else { return Err(e) };
            let dev = Device { ip: found.ip, ..dev };
            {
                let mut cfg = state.cfg.lock().unwrap();
                cfg.device = Some(dev.clone());
                config::save(app, &cfg)?;
            }
            let _ = app.emit("ac-info", info(&state.cfg.lock().unwrap()));
            op(dev).await
        }
        r => r,
    }
}

fn publish(app: &AppHandle, snap: Snapshot) {
    tray::set_tooltip(app, &snap);
    *app.state::<AppState>().last.lock().unwrap() = Some(snap.clone());
    let _ = app.emit("ac-status", snap);
}

pub async fn poll(app: &AppHandle) -> Result<Snapshot, String> {
    let res = with_device(app, |d| async move { d.status(STATUS_COLS).await }).await;
    let status = match res {
        Ok(s) => s,
        Err(e) => {
            tray::set_tooltip_text(app, "Ar-condicionado · sem conexão");
            let _ = app.emit("ac-error", &e);
            return Err(e);
        }
    };
    let room = device::room_temp(&status, &mut app.state::<AppState>().v4.lock().unwrap());
    let props = status.iter().filter_map(|(k, v)| v.as_i64().map(|n| (k.clone(), n))).collect();
    let snap = Snapshot { props, room };
    publish(app, snap.clone());
    Ok(snap)
}

/// Sends a cmd and folds the new values into the cached snapshot.
pub async fn apply(app: &AppHandle, props: Vec<(String, i64)>) -> Result<(), String> {
    let p = props.clone();
    with_device(app, move |d| {
        let p = p.clone();
        async move { d.cmd(&p).await }
    })
    .await?;
    let last = app.state::<AppState>().last.lock().unwrap().clone();
    if let Some(mut snap) = last {
        snap.props.extend(props);
        publish(app, snap);
    }
    Ok(())
}

pub async fn toggle_power(app: &AppHandle) -> Result<(), String> {
    let on = app.state::<AppState>().last.lock().unwrap().as_ref().is_some_and(|s| s.props.get("Pow") == Some(&1));
    apply(app, vec![("Pow".into(), if on { 0 } else { 1 })]).await
}

/// Polls every 5s while the flyout is visible, every 60s while hidden.
pub fn spawn_poller(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let state = app.state::<AppState>();
            if state.cfg.lock().unwrap().device.is_some() {
                let _ = poll(&app).await;
            }
            let visible = app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
            let wait = Duration::from_secs(if visible { 5 } else { 60 });
            tokio::select! {
                _ = tokio::time::sleep(wait) => {}
                _ = state.wake.notified() => {}
            }
        }
    });
}

#[tauri::command]
pub fn get_info(state: State<AppState>) -> Option<Info> {
    info(&state.cfg.lock().unwrap())
}

#[tauri::command]
pub fn get_snapshot(state: State<AppState>) -> Option<Snapshot> {
    state.last.lock().unwrap().clone()
}

#[tauri::command]
pub async fn scan() -> Result<Vec<Found>, String> {
    discovery::scan(SCAN_WAIT).await
}

#[tauri::command]
pub async fn bind(app: AppHandle, found: Found) -> Result<Info, String> {
    let state = app.state::<AppState>();
    let dev = {
        let _io = state.io.lock().await;
        device::bind(found.ip, &found.mac, found.version).await?
    };
    let cfg = Config { name: found.name, device: Some(dev) };
    config::save(&app, &cfg)?;
    let out = info(&cfg);
    *state.cfg.lock().unwrap() = cfg;
    *state.v4.lock().unwrap() = None;
    *state.last.lock().unwrap() = None;
    state.wake.notify_one();
    Ok(out.unwrap())
}

#[tauri::command]
pub async fn refresh(app: AppHandle) -> Result<Snapshot, String> {
    poll(&app).await
}

#[tauri::command]
pub async fn send(app: AppHandle, props: Vec<(String, i64)>) -> Result<(), String> {
    apply(&app, props).await
}

/// First run: the page asks to be shown once it has loaded. Showing it from setup()
/// lost focus to the WebView2 init and the blur handler hid it again.
#[tauri::command]
pub fn show(app: AppHandle) {
    tray::show_flyout(&app, None);
}

#[tauri::command]
pub fn rename(app: AppHandle, state: State<AppState>, name: String) -> Result<(), String> {
    let mut cfg = state.cfg.lock().unwrap();
    cfg.name = name.trim().to_string();
    config::save(&app, &cfg)
}
