use btlock_bt::{AssociationInfo, BtDevice};
use btlock_core::AppConfig;
use btlock_engine::{
    engine_associate_saved, engine_get_config, engine_get_status, engine_save_config,
    engine_scan, engine_select_device, engine_set_session_locked_hook, engine_start_monitoring,
    engine_stop_monitoring, new_engine, SharedEngine,
};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};

type SharedState = SharedEngine;

#[tauri::command]
fn get_config(state: State<'_, SharedState>) -> Result<AppConfig, String> {
    Ok(engine_get_config(state.inner()))
}

#[tauri::command]
fn save_app_config(config: AppConfig, state: State<'_, SharedState>) -> Result<(), String> {
    engine_save_config(state.inner(), config)
}

#[tauri::command]
fn get_status(state: State<'_, SharedState>) -> Result<btlock_core::MonitorStatus, String> {
    Ok(engine_get_status(state.inner()))
}

#[tauri::command]
async fn scan_devices() -> Result<Vec<BtDevice>, String> {
    engine_scan().await
}

#[tauri::command]
async fn select_device(
    device: BtDevice,
    state: State<'_, SharedState>,
) -> Result<AppConfig, String> {
    engine_select_device(state.inner(), device).await
}

#[tauri::command]
async fn associate_saved_device(state: State<'_, SharedState>) -> Result<AssociationInfo, String> {
    engine_associate_saved(state.inner()).await
}

#[tauri::command]
async fn start_monitoring(app: AppHandle, state: State<'_, SharedState>) -> Result<(), String> {
    let shared = state.inner().clone();
    engine_set_session_locked_hook(
        &shared,
        Some(Arc::new(move || {
            let _ = app.emit("session-locked", ());
        })),
    );
    engine_start_monitoring(&shared).await
}

#[tauri::command]
fn stop_monitoring(state: State<'_, SharedState>) -> Result<(), String> {
    engine_stop_monitoring(state.inner())
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
            app.manage(new_engine());
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
