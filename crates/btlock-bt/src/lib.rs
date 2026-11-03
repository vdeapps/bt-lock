//! BlueZ-backed BLE discovery and RSSI reads.

use btlock_core::DeviceKind;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtDevice {
    pub address: String,
    pub name: String,
    pub rssi: Option<i16>,
    pub kind: DeviceKind,
}

#[derive(Debug, Error)]
pub enum BtError {
    #[error("bluetooth: {0}")]
    Bluez(String),
}

fn guess_kind(name: &str) -> DeviceKind {
    let n = name.to_lowercase();
    if n.contains("watch") || n.contains("montre") || n.contains("band") || n.contains("fit") {
        DeviceKind::Watch
    } else if n.contains("phone")
        || n.contains("iphone")
        || n.contains("pixel")
        || n.contains("galaxy")
        || n.contains("téléphone")
        || n.contains("telephone")
    {
        DeviceKind::Phone
    } else {
        DeviceKind::Unknown
    }
}

pub async fn scan_devices(duration: Duration) -> Result<Vec<BtDevice>, BtError> {
    let session = bluer::Session::new()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;
    let adapter = session
        .default_adapter()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;
    adapter
        .set_powered(true)
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;

    let _discover = adapter
        .discover_devices()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;

    tokio::time::sleep(duration).await;

    let addrs = adapter
        .device_addresses()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;

    let mut list = Vec::new();
    for addr in addrs {
        let device = adapter
            .device(addr)
            .map_err(|e| BtError::Bluez(e.to_string()))?;
        let address = addr.to_string();
        let name = device
            .name()
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?
            .unwrap_or_else(|| "Appareil inconnu".into());
        let rssi = device
            .rssi()
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?;
        let kind = guess_kind(&name);
        list.push(BtDevice {
            address,
            name,
            rssi,
            kind,
        });
    }

    list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(list)
}

pub async fn read_rssi(address: &str) -> Result<Option<i16>, BtError> {
    let session = bluer::Session::new()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;
    let adapter = session
        .default_adapter()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::Bluez(format!("invalid address {address}")))?;
    let device = adapter
        .device(addr)
        .map_err(|e| BtError::Bluez(e.to_string()))?;
    device
        .rssi()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guess_watch_and_phone() {
        assert_eq!(guess_kind("Galaxy Watch 6"), DeviceKind::Watch);
        assert_eq!(guess_kind("iPhone de Alex"), DeviceKind::Phone);
    }
}
