//! Configuration, persistence helpers, and lock policy for BTlock.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockSettings {
    pub delay_before_lock_secs: u32,
    pub max_attempts: u32,
    pub rssi_threshold_dbm: i16,
    pub lock_command: String,
}

impl Default for LockSettings {
    fn default() -> Self {
        Self {
            delay_before_lock_secs: 5,
            max_attempts: 3,
            rssi_threshold_dbm: -80,
            lock_command: "loginctl lock-session".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedDevice {
    pub address: String,
    pub name: String,
    pub kind: DeviceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Phone,
    Watch,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub settings: LockSettings,
    pub device: Option<SelectedDevice>,
    pub monitoring_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub active: bool,
    pub last_rssi: Option<i16>,
    pub weak_streak: u32,
    pub last_event: String,
}

impl Default for MonitorStatus {
    fn default() -> Self {
        Self {
            active: false,
            last_rssi: None,
            weak_streak: 0,
            last_event: "Inactif".into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("btlock")
        .join("config.json")
}

pub fn load_config(path: &Path) -> Result<AppConfig, ConfigError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn save_config(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// État du signal pour la politique de verrouillage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalState {
    Strong,
    Weak,
    Unknown,
}

/// Returns true when signal is considered too weak (device "far").
pub fn is_signal_weak(rssi: i16, threshold: i16) -> bool {
    rssi < threshold
}

/// Une mesure absente n'est pas un signal faible (évite les faux verrouillages).
pub fn classify_signal(rssi: Option<i16>, threshold: i16) -> SignalState {
    match rssi {
        Some(v) if v < threshold => SignalState::Weak,
        Some(_) => SignalState::Strong,
        None => SignalState::Unknown,
    }
}

/// Lock the graphical session using the configured shell command.
pub fn lock_session(settings: &LockSettings) -> Result<(), ConfigError> {
    let parts: Vec<&str> = settings.lock_command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty lock command",
        )));
    }
    let status = std::process::Command::new(parts[0])
        .args(&parts[1..])
        .status()?;
    if !status.success() {
        return Err(ConfigError::Io(std::io::Error::other(format!(
            "lock command failed: {status}"
        ))));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_sane() {
        let s = LockSettings::default();
        assert!(s.delay_before_lock_secs > 0);
        assert!(s.rssi_threshold_dbm < 0);
    }

    #[test]
    fn unknown_signal_is_not_weak() {
        assert_eq!(classify_signal(None, -70), SignalState::Unknown);
    }

    #[test]
    fn weak_signal_below_threshold() {
        assert!(is_signal_weak(-85, -70));
        assert!(!is_signal_weak(-60, -70));
    }

    #[test]
    fn roundtrip_config_json() {
        let cfg = AppConfig {
            device: Some(SelectedDevice {
                address: "AA:BB:CC:DD:EE:FF".into(),
                name: "Mon téléphone".into(),
                kind: DeviceKind::Phone,
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }
}
