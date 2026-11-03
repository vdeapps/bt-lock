//! Session BlueZ maintenue (discovery + agent) pour un appareil associé.

use crate::{guess_kind, BtDevice, BtError};
use bluer::agent::Agent;
use bluer::{Address, DiscoveryFilter, DiscoveryTransport};
use futures::StreamExt;
use std::time::Duration;

/// Résultat de l’association (appairage / confiance).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssociationInfo {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub trusted: bool,
    pub message: String,
}

pub struct ActiveLink {
    _session: bluer::Session,
    _agent: bluer::agent::AgentHandle,
    _discovery_task: tokio::task::JoinHandle<()>,
    adapter: bluer::Adapter,
    address: Address,
}

impl ActiveLink {
    /// Lance la discovery, matérialise l’appareil, appaire si besoin et marque comme trusted.
    pub async fn associate(address: &str) -> Result<(Self, AssociationInfo), BtError> {
        let addr: Address = address
            .parse()
            .map_err(|_| BtError::Bluez(format!("adresse invalide : {address}")))?;

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

        let agent = Agent {
            request_default: true,
            ..Default::default()
        };
        let agent_handle = session
            .register_agent(agent)
            .await
            .map_err(|e| BtError::Bluez(format!("agent BT : {e}")))?;

        adapter
            .set_discovery_filter(DiscoveryFilter {
                transport: DiscoveryTransport::Auto,
                ..Default::default()
            })
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?;

        let adapter_bg = adapter.clone();
        let discovery_task = tokio::spawn(async move {
            if let Ok(mut stream) = adapter_bg.discover_devices().await {
                while stream.next().await.is_some() {}
            }
        });

        Self::wait_for_device(&adapter, addr, Duration::from_secs(20)).await?;

        let device = adapter
            .device(addr)
            .map_err(|e| BtError::Bluez(e.to_string()))?;

        let name = device
            .name()
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?
            .unwrap_or_else(|| "Appareil inconnu".into());

        let mut paired = device
            .is_paired()
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?;

        if !paired {
            device.pair().await.map_err(|e| {
                BtError::Bluez(format!("appairage : {e} — acceptez la demande sur le téléphone"))
            })?;
            paired = true;
        }

        device
            .set_trusted(true)
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?;

        let trusted = device
            .is_trusted()
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))?;

        let _ = device.connect().await;

        let message = if trusted {
            format!("{name} associé — discovery active pour le RSSI.")
        } else {
            format!("{name} appairé ; vérifiez la confiance dans les paramètres BT.")
        };

        let info = AssociationInfo {
            address: addr.to_string(),
            name: name.clone(),
            paired,
            trusted,
            message,
        };

        Ok((
            Self {
                _session: session,
                _agent: agent_handle,
                _discovery_task: discovery_task,
                adapter,
                address: addr,
            },
            info,
        ))
    }

    async fn wait_for_device(
        adapter: &bluer::Adapter,
        addr: Address,
        timeout: Duration,
    ) -> Result<(), BtError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let addrs = adapter
                .device_addresses()
                .await
                .map_err(|e| BtError::Bluez(e.to_string()))?;
            if addrs.contains(&addr) {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(BtError::Bluez(format!(
                    "appareil {addr} introuvable après {} s — Bluetooth activé sur l’appareil, puis Rechercher",
                    timeout.as_secs()
                )));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    pub async fn read_rssi(&self) -> Result<Option<i16>, BtError> {
        let device = self
            .adapter
            .device(self.address)
            .map_err(|e| BtError::Bluez(e.to_string()))?;
        device
            .rssi()
            .await
            .map_err(|e| BtError::Bluez(e.to_string()))
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

    adapter
        .set_discovery_filter(DiscoveryFilter {
            transport: DiscoveryTransport::Auto,
            ..Default::default()
        })
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;

    let mut events = adapter
        .discover_devices_with_changes()
        .await
        .map_err(|e| BtError::Bluez(e.to_string()))?;

    let deadline = tokio::time::Instant::now() + duration;
    let mut map: std::collections::HashMap<String, BtDevice> = std::collections::HashMap::new();

    while tokio::time::Instant::now() < deadline {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(300)) => {},
            evt = events.next() => {
                if let Some(bluer::AdapterEvent::DeviceAdded(addr)) = evt {
                    if let Ok(dev) = adapter.device(addr) {
                        let address = addr.to_string();
                        let name = dev.name().await.ok().flatten().unwrap_or_else(|| "Appareil inconnu".into());
                        let rssi = dev.rssi().await.ok().flatten();
                        let kind = guess_kind(&name);
                        map.insert(address.clone(), BtDevice { address, name, rssi, kind });
                    }
                }
            }
        }
    }

    let mut list: Vec<BtDevice> = map.into_values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(list)
}

pub async fn read_rssi(address: &str) -> Result<Option<i16>, BtError> {
    let (link, _) = ActiveLink::associate(address).await?;
    link.read_rssi().await
}
