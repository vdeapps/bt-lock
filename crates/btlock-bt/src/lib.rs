//! BlueZ-backed BLE discovery and RSSI reads.

mod link;

use btlock_core::DeviceKind;
pub use link::{ActiveLink, AssociationInfo};
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

pub(crate) fn guess_kind(name: &str) -> DeviceKind {
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
    link::scan_devices(duration).await
}

pub async fn associate_device(address: &str) -> Result<(ActiveLink, AssociationInfo), BtError> {
    ActiveLink::associate(address).await
}

pub async fn read_rssi(address: &str) -> Result<Option<i16>, BtError> {
    link::read_rssi(address).await
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
