use btlock_bt::{scan_devices as bt_scan, associate_device, AssociationInfo, BtDevice};
use btlock_core::{
    config_path, is_signal_weak, load_config, lock_session, save_config, AppConfig, MonitorStatus,
    SelectedDevice,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

struct MonitorHandle {
    stop: Arc<Notify>,
    _join: JoinHandle<()>,
}

struct AppState {
    config: AppConfig,
    config_path: PathBuf,
    status: MonitorStatus,
    monitor: Option<MonitorHandle>,
    active_link: Option<Arc<btlock_bt::ActiveLink>>,
}

impl AppState {
    fn new() -> Self {
        let path = config_path();
        let config = load_config(&path).unwrap_or_default();
        Self {
            config,
            config_path: path,
            status: MonitorStatus::default(),
            monitor: None,
            active_link: None,
        }
    }

    fn persist(&self) -> Result<(), String> {
        save_config(&self.config_path, &self.config).map_err(|e| e.to_string())
    }
}

type SharedState = Arc<Mutex<AppState>>;

#[tauri::command]
fn get_config(state: State<'_, SharedState>) -> Result<AppConfig, String> {
    Ok(state.blocking_lock().config.clone())
}

#[tauri::command]
fn save_app_config(config: AppConfig, state: State<'_, SharedState>) -> Result<(), String> {
    let mut guard = state.blocking_lock();
    guard.config = config;
    guard.persist()
}

#[tauri::command]
fn get_status(state: State<'_, SharedState>) -> Result<MonitorStatus, String> {
    Ok(state.blocking_lock().status.clone())
}

#[tauri::command]
async fn scan_devices() -> Result<Vec<BtDevice>, String> {
    bt_scan(Duration::from_secs(8))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn select_device(
    device: BtDevice,
    state: State<'_, SharedState>,
) -> Result<AppConfig, String> {
    let (link, info) = associate_device(&device.address)
        .await
        .map_err(|e| e.to_string())?;
    let mut guard = state.lock().await;
    guard.active_link = Some(Arc::new(link));
    guard.config.device = Some(SelectedDevice {
        address: device.address,
        name: device.name,
        kind: device.kind,
    });
    guard.status.last_event = info.message;
    guard.persist()?;
    Ok(guard.config.clone())
}

#[tauri::command]
async fn associate_saved_device(state: State<'_, SharedState>) -> Result<AssociationInfo, String> {
    let address = {
        let guard = state.lock().await;
        guard
            .config
            .device
            .as_ref()
            .map(|d| d.address.clone())
            .ok_or_else(|| "Aucun appareil sélectionné.".to_string())?
    };
    let (link, info) = associate_device(&address)
        .await
        .map_err(|e| e.to_string())?;
    let mut guard = state.lock().await;
    guard.active_link = Some(Arc::new(link));
    guard.status.last_event = info.message.clone();
    Ok(info)
}

fn stop_monitor_task(guard: &mut AppState) {
    if let Some(handle) = guard.monitor.take() {
        handle.stop.notify_one();
    }
    guard.status.active = false;
    guard.config.monitoring_enabled = false;
}

fn stop_monitor(guard: &mut AppState) {
    stop_monitor_task(guard);
    guard.status.last_event = "Surveillance arrêtée".into();
    guard.active_link = None;
    let _ = guard.persist();
}

async fn run_monitor(app: AppHandle, shared: SharedState, stop: Arc<Notify>) {
    loop {
        tokio::select! {
            _ = stop.notified() => break,
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }

        let (_device, settings) = {
            let guard = shared.lock().await;
            let Some(device) = guard.config.device.clone() else {
                continue;
            };
            (device, guard.config.settings.clone())
        };

        let link = {
            let guard = shared.lock().await;
            guard.active_link.clone()
        };
        let Some(link) = link else {
            let mut guard = shared.lock().await;
            guard.status.last_event =
                "Appareil non associé — resélectionnez après Rechercher".into();
            continue;
        };

        let rssi = match link.read_rssi().await {
            Ok(v) => v,
            Err(e) => {
                let mut guard = shared.lock().await;
                guard.status.last_event = format!("Erreur BT : {e}");
                continue;
            }
        };

        let should_lock = {
            let mut guard = shared.lock().await;
            guard.status.last_rssi = rssi;
            let weak = rssi
                .map(|v| is_signal_weak(v, settings.rssi_threshold_dbm))
                .unwrap_or(true);

            if weak {
                guard.status.weak_streak = guard.status.weak_streak.saturating_add(1);
                guard.status.last_event = format!(
                    "Signal faible ({}/{})",
                    guard.status.weak_streak, settings.max_attempts
                );
                guard.status.weak_streak >= settings.max_attempts
            } else {
                guard.status.weak_streak = 0;
                guard.status.last_event = format!("Signal OK ({rssi:?} dBm)");
                false
            }
        };

        if should_lock {
            {
                let mut guard = shared.lock().await;
                guard.status.last_event = format!(
                    "Verrouillage dans {} s…",
                    settings.delay_before_lock_secs
                );
            }
            tokio::select! {
                _ = stop.notified() => break,
                _ = tokio::time::sleep(Duration::from_secs(settings.delay_before_lock_secs as u64)) => {}
            }
            let mut guard = shared.lock().await;
            if let Err(e) = lock_session(&settings) {
                guard.status.last_event = format!("Échec verrouillage : {e}");
            } else {
                guard.status.last_event = "Session verrouillée".into();
                guard.status.weak_streak = 0;
                let _ = app.emit("session-locked", ());
            }
        }
    }
}

#[tauri::command]
async fn start_monitoring(app: AppHandle, state: State<'_, SharedState>) -> Result<(), String> {
    {
        let guard = state.lock().await;
        if guard.config.device.is_none() {
            return Err("Choisissez un appareil Bluetooth d'abord.".into());
        }
        if guard.active_link.is_none() {
            let address = guard.config.device.as_ref().unwrap().address.clone();
            drop(guard);
            let (link, info) = associate_device(&address)
                .await
                .map_err(|e| e.to_string())?;
            let mut guard = state.lock().await;
            guard.active_link = Some(Arc::new(link));
            guard.status.last_event = info.message;
        }
    }

    let mut guard = state.lock().await;
    stop_monitor_task(&mut guard);
    guard.config.monitoring_enabled = true;
    guard.status = MonitorStatus {
        active: true,
        last_event: "Surveillance démarrée".into(),
        ..Default::default()
    };
    guard.persist()?;

    let shared = state.inner().clone();
    let stop = Arc::new(Notify::new());
    let stop_clone = stop.clone();
    let join = tokio::spawn(run_monitor(app, shared, stop_clone));
    guard.monitor = Some(MonitorHandle { stop, _join: join });
    Ok(())
}

#[tauri::command]
fn stop_monitoring(state: State<'_, SharedState>) -> Result<(), String> {
    let mut guard = state.blocking_lock();
    stop_monitor(&mut guard);
    Ok(())
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Ouvrir BTlock", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("BTlock — verrouillage Bluetooth")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            build_tray(app.handle())?;
            let shared: SharedState = Arc::new(Mutex::new(AppState::new()));
            app.manage(shared);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_app_config,
            get_status,
            scan_devices,
            select_device,
            associate_saved_device,
            start_monitoring,
            stop_monitoring,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
