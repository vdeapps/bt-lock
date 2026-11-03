//! Logique applicative BTlock (config, surveillance, verrouillage) sans UI.

use btlock_bt::{associate_device, scan_devices as bt_scan, ActiveLink, AssociationInfo, BtDevice};
use btlock_core::{
    classify_link_presence, classify_signal, config_path, load_config, lock_session, save_config,
    AppConfig, MonitorStatus, ProximityMethod, SelectedDevice, SignalState,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

struct MonitorHandle {
    stop: Arc<Notify>,
    _join: JoinHandle<()>,
}

pub type SessionLockedHook = Arc<dyn Fn() + Send + Sync>;

pub struct EngineState {
    pub config: AppConfig,
    config_path: PathBuf,
    pub status: MonitorStatus,
    monitor: Option<MonitorHandle>,
    active_link: Option<Arc<ActiveLink>>,
    on_session_locked: Option<SessionLockedHook>,
}

pub fn engine_set_session_locked_hook(shared: &SharedEngine, hook: Option<SessionLockedHook>) {
    shared.blocking_lock().on_session_locked = hook;
}

impl EngineState {
    fn new() -> Self {
        let path = config_path();
        let config = load_config(&path).unwrap_or_default();
        Self {
            config,
            config_path: path,
            status: MonitorStatus::default(),
            monitor: None,
            active_link: None,
            on_session_locked: None,
        }
    }

    fn persist(&self) -> Result<(), String> {
        save_config(&self.config_path, &self.config).map_err(|e| e.to_string())
    }
}

pub type SharedEngine = Arc<Mutex<EngineState>>;

pub fn new_engine() -> SharedEngine {
    Arc::new(Mutex::new(EngineState::new()))
}

pub async fn engine_scan() -> Result<Vec<BtDevice>, String> {
    bt_scan(Duration::from_secs(8))
        .await
        .map_err(|e| e.to_string())
}

pub async fn engine_select_device(
    shared: &SharedEngine,
    device: BtDevice,
) -> Result<AppConfig, String> {
    let (link, info) = associate_device(&device.address)
        .await
        .map_err(|e| e.to_string())?;
    let mut guard = shared.lock().await;
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

pub async fn engine_associate_saved(
    shared: &SharedEngine,
) -> Result<AssociationInfo, String> {
    let address = {
        let guard = shared.lock().await;
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
    let mut guard = shared.lock().await;
    guard.active_link = Some(Arc::new(link));
    guard.status.last_event = info.message.clone();
    Ok(info)
}

fn stop_monitor_task(guard: &mut EngineState) {
    if let Some(handle) = guard.monitor.take() {
        handle.stop.notify_one();
    }
    guard.status.active = false;
    guard.config.monitoring_enabled = false;
}

pub fn engine_stop_monitoring(shared: &SharedEngine) -> Result<(), String> {
    let mut guard = shared.blocking_lock();
    stop_monitor_task(&mut guard);
    guard.status.last_event = "Surveillance arrêtée".into();
    guard.active_link = None;
    guard.persist()
}

async fn run_monitor(shared: SharedEngine, stop: Arc<Notify>) {
    loop {
        tokio::select! {
            _ = stop.notified() => break,
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }

        let settings = {
            let guard = shared.lock().await;
            if guard.config.device.is_none() {
                continue;
            }
            guard.config.settings.clone()
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

        let rssi = match link.read_fresh_rssi().await {
            Ok(v) => v,
            Err(e) => {
                let mut guard = shared.lock().await;
                guard.status.last_event = format!("Erreur BT : {e}");
                continue;
            }
        };

        let (signal, link_connected) = match settings.proximity_method {
            ProximityMethod::LinkPresence => {
                let connected = match link.presence_via_link().await {
                    Ok(v) => v,
                    Err(e) => {
                        let mut guard = shared.lock().await;
                        guard.status.last_event = format!("Erreur BT : {e}");
                        continue;
                    }
                };
                (classify_link_presence(connected), Some(connected))
            }
            ProximityMethod::Rssi => (classify_signal(rssi, settings.rssi_threshold_dbm), None),
        };

        let should_lock = {
            let mut guard = shared.lock().await;
            guard.status.last_rssi = rssi;
            guard.status.link_connected = link_connected;
            match signal {
                SignalState::Weak => {
                    guard.status.weak_streak = guard.status.weak_streak.saturating_add(1);
                    guard.status.last_event = match settings.proximity_method {
                        ProximityMethod::LinkPresence => format!(
                            "Téléphone déconnecté ({}/{})",
                            guard.status.weak_streak,
                            settings.max_attempts,
                        ),
                        ProximityMethod::Rssi => format!(
                            "Signal faible ({}/{}) — {rssi:?} dBm, seuil {} dBm",
                            guard.status.weak_streak,
                            settings.max_attempts,
                            settings.rssi_threshold_dbm
                        ),
                    };
                    guard.status.weak_streak >= settings.max_attempts
                }
                SignalState::Strong => {
                    guard.status.weak_streak = 0;
                    guard.status.last_event = match settings.proximity_method {
                        ProximityMethod::LinkPresence => {
                            if let Some(r) = rssi {
                                format!("Téléphone connecté ({r} dBm)")
                            } else {
                                "Téléphone connecté".into()
                            }
                        }
                        ProximityMethod::Rssi => format!("Signal OK ({rssi:?} dBm)"),
                    };
                    false
                }
                SignalState::Unknown => {
                    guard.status.last_event = match settings.proximity_method {
                        ProximityMethod::Rssi => {
                            "Mesure RSSI expirée ou indisponible — pas de verrouillage".into()
                        }
                        ProximityMethod::LinkPresence => {
                            "État connexion indéterminé — pas de verrouillage".into()
                        }
                    };
                    false
                }
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
            let confirm_signal = match settings.proximity_method {
                ProximityMethod::LinkPresence => {
                    classify_link_presence(link.presence_via_link().await.unwrap_or(false))
                }
                ProximityMethod::Rssi => {
                    let confirm = link.read_fresh_rssi().await.ok().flatten();
                    classify_signal(confirm, settings.rssi_threshold_dbm)
                }
            };
            let mut locked_hook = None;
            match confirm_signal {
                SignalState::Weak => {
                    let mut guard = shared.lock().await;
                    if let Err(e) = lock_session(&settings) {
                        guard.status.last_event = format!("Échec verrouillage : {e}");
                    } else {
                        guard.status.last_event = "Session verrouillée".into();
                        guard.status.weak_streak = 0;
                        locked_hook = guard.on_session_locked.clone();
                    }
                }
                SignalState::Strong => {
                    let mut guard = shared.lock().await;
                    guard.status.weak_streak = 0;
                    guard.status.last_event = match settings.proximity_method {
                        ProximityMethod::LinkPresence => {
                            "Verrouillage annulé — téléphone reconnecté".into()
                        }
                        ProximityMethod::Rssi => format!(
                            "Verrouillage annulé — signal OK ({:?} dBm)",
                            link.read_fresh_rssi().await.ok().flatten()
                        ),
                    };
                }
                SignalState::Unknown => {
                    let mut guard = shared.lock().await;
                    guard.status.weak_streak = 0;
                    guard.status.last_event = match settings.proximity_method {
                        ProximityMethod::LinkPresence => {
                            "Verrouillage annulé — connexion incertaine".into()
                        }
                        ProximityMethod::Rssi => {
                            "Verrouillage annulé — mesure RSSI expirée".into()
                        }
                    };
                }
            }
            if let Some(h) = locked_hook {
                h();
            }
        }
    }
}

pub async fn engine_start_monitoring(shared: &SharedEngine) -> Result<(), String> {
    {
        let guard = shared.lock().await;
        if guard.config.device.is_none() {
            return Err("Choisissez un appareil Bluetooth d'abord.".into());
        }
        if guard.active_link.is_none() {
            let address = guard.config.device.as_ref().unwrap().address.clone();
            drop(guard);
            let (link, info) = associate_device(&address)
                .await
                .map_err(|e| e.to_string())?;
            let mut guard = shared.lock().await;
            guard.active_link = Some(Arc::new(link));
            guard.status.last_event = info.message;
        }
    }

    let (link_arc, method) = {
        let guard = shared.lock().await;
        (
            guard.active_link.clone(),
            guard.config.settings.proximity_method,
        )
    };
    if let Some(link) = link_arc {
        link.prepare_for_monitoring(method)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut guard = shared.lock().await;
    stop_monitor_task(&mut guard);
    guard.config.monitoring_enabled = true;
    guard.status = MonitorStatus {
        active: true,
        last_event: match method {
            ProximityMethod::LinkPresence => {
                "Surveillance active — présence par connexion Bluetooth".into()
            }
            ProximityMethod::Rssi => {
                "Surveillance active — distance RSSI (mesures < 3 s)".into()
            }
        },
        ..Default::default()
    };
    guard.persist()?;

    let shared_clone = Arc::clone(shared);
    let stop = Arc::new(Notify::new());
    let stop_clone = stop.clone();
    let join = tokio::spawn(run_monitor(shared_clone, stop_clone));
    guard.monitor = Some(MonitorHandle { stop, _join: join });
    Ok(())
}

pub fn engine_save_config(shared: &SharedEngine, config: AppConfig) -> Result<(), String> {
    let mut guard = shared.blocking_lock();
    guard.config = config;
    guard.persist()
}

pub fn engine_get_config(shared: &SharedEngine) -> AppConfig {
    shared.blocking_lock().config.clone()
}

pub fn engine_get_status(shared: &SharedEngine) -> MonitorStatus {
    shared.blocking_lock().status.clone()
}
