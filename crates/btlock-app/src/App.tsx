import { useCallback, useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import "./App.css";

type DeviceKind = "phone" | "watch" | "unknown";

interface LockSettings {
  delay_before_lock_secs: number;
  max_attempts: number;
  rssi_threshold_dbm: number;
  lock_command: string;
}

interface SelectedDevice {
  address: string;
  name: string;
  kind: DeviceKind;
}

interface AppConfig {
  settings: LockSettings;
  device: SelectedDevice | null;
  monitoring_enabled: boolean;
}

interface BtDevice {
  address: string;
  name: string;
  rssi: number | null;
  kind: DeviceKind;
}

interface MonitorStatus {
  active: boolean;
  last_rssi: number | null;
  weak_streak: number;
  last_event: string;
}

const kindLabel: Record<DeviceKind, string> = {
  phone: "Téléphone",
  watch: "Montre",
  unknown: "Autre",
};

const demoConfig: AppConfig = {
  settings: {
    delay_before_lock_secs: 5,
    max_attempts: 3,
    rssi_threshold_dbm: -80,
    lock_command: "loginctl lock-session",
  },
  device: {
    address: "AA:BB:CC:11:22:33",
    name: "iPhone (démo)",
    kind: "phone",
  },
  monitoring_enabled: false,
};

const demoDevices: BtDevice[] = [
  {
    address: "AA:BB:CC:11:22:33",
    name: "iPhone (démo)",
    rssi: -58,
    kind: "phone",
  },
  {
    address: "BB:CC:DD:44:55:66",
    name: "Galaxy Watch (démo)",
    rssi: -72,
    kind: "watch",
  },
];

const demoStatus: MonitorStatus = {
  active: false,
  last_rssi: -58,
  weak_streak: 0,
  last_event: "Mode aperçu navigateur",
};

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [devices, setDevices] = useState<BtDevice[]>([]);
  const [status, setStatus] = useState<MonitorStatus | null>(null);
  const [scanning, setScanning] = useState(false);
  const [message, setMessage] = useState<string>("");

  const refreshConfig = useCallback(async () => {
    if (!isTauri()) {
      setConfig(demoConfig);
      return;
    }
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  const refreshStatus = useCallback(async () => {
    if (!isTauri()) {
      setStatus(demoStatus);
      return;
    }
    const st = await invoke<MonitorStatus>("get_status");
    setStatus(st);
  }, []);

  useEffect(() => {
    refreshConfig();
    refreshStatus();
    const id = window.setInterval(refreshStatus, 2000);
    return () => window.clearInterval(id);
  }, [refreshConfig, refreshStatus]);

  const updateSettings = async (patch: Partial<LockSettings>) => {
    if (!config) return;
    const next = {
      ...config,
      settings: { ...config.settings, ...patch },
    };
    await invoke("save_app_config", { config: next });
    setConfig(next);
    setMessage("Paramètres enregistrés.");
  };

  const runScan = async () => {
    setScanning(true);
    setMessage("Recherche des appareils Bluetooth (≈8 s)…");
    try {
      if (!isTauri()) {
        await new Promise((r) => setTimeout(r, 800));
        setDevices(demoDevices);
        setMessage(`${demoDevices.length} appareil(s) trouvé(s) (démo).`);
        return;
      }
      const list = await invoke<BtDevice[]>("scan_devices");
      setDevices(list);
      setMessage(`${list.length} appareil(s) trouvé(s).`);
    } catch (e) {
      setMessage(String(e));
    } finally {
      setScanning(false);
    }
  };

  const pickDevice = async (device: BtDevice) => {
    setMessage(`Association avec ${device.name}… (acceptez l’appairage sur l’appareil si demandé)`);
    try {
      const cfg = await invoke<AppConfig>("select_device", { device });
      setConfig(cfg);
      setMessage(`${device.name} associé — vous pouvez démarrer la surveillance.`);
    } catch (e) {
      setMessage(String(e));
    }
  };

  const toggleMonitor = async () => {
    if (!config) return;
    try {
      if (status?.active) {
        await invoke("stop_monitoring");
        setMessage("Surveillance arrêtée.");
      } else {
        await invoke("start_monitoring");
        setMessage("Surveillance active — l’app reste dans la barre système.");
      }
      await refreshStatus();
      await refreshConfig();
    } catch (e) {
      setMessage(String(e));
    }
  };

  if (!config) {
    return (
      <div className="shell loading">
        <div className="pulse" />
        <p>Chargement…</p>
      </div>
    );
  }

  return (
    <div className="shell">
      <header className="hero">
        <div className="brand">
          <span className="logo" aria-hidden>
            ◉
          </span>
          <div>
            <h1>BTlock</h1>
            <p>Verrouillez votre session quand votre téléphone ou montre s’éloigne.</p>
          </div>
        </div>
        <div className={`pill ${status?.active ? "on" : "off"}`}>
          {status?.active ? "Surveillance active" : "Inactif"}
        </div>
      </header>

      {message && <div className="toast">{message}</div>}

      <main className="grid">
        <section className="card">
          <h2>Appareils</h2>
          <p className="hint">
            Activez le Bluetooth sur votre téléphone ou montre et lancez une recherche.
          </p>
          <button className="btn primary" onClick={runScan} disabled={scanning}>
            {scanning ? "Recherche…" : "Rechercher"}
          </button>
          <ul className="device-list">
            {devices.map((d) => (
              <li key={d.address}>
                <button
                  type="button"
                  className={
                    config.device?.address === d.address ? "device selected" : "device"
                  }
                  onClick={() => pickDevice(d)}
                >
                  <span className="device-name">{d.name}</span>
                  <span className="device-meta">
                    {kindLabel[d.kind]} · {d.address}
                    {d.rssi != null ? ` · ${d.rssi} dBm` : ""}
                  </span>
                </button>
              </li>
            ))}
            {!devices.length && !scanning && (
              <li className="empty">Aucun appareil — lancez une recherche.</li>
            )}
          </ul>
        </section>

        <section className="card">
          <h2>Paramètres</h2>
          <label>
            Délai avant verrouillage (s)
            <input
              type="range"
              min={1}
              max={60}
              value={config.settings.delay_before_lock_secs}
              onChange={(e) =>
                updateSettings({ delay_before_lock_secs: Number(e.target.value) })
              }
            />
            <strong>{config.settings.delay_before_lock_secs} s</strong>
          </label>
          <label>
            Lectures faibles consécutives
            <input
              type="range"
              min={1}
              max={10}
              value={config.settings.max_attempts}
              onChange={(e) =>
                updateSettings({ max_attempts: Number(e.target.value) })
              }
            />
            <strong>{config.settings.max_attempts}</strong>
          </label>
          <label>
            Seuil RSSI (verrouiller si mesure sous ce dBm — ex. -80 loin, -56 ~5 m)
            <input
              type="range"
              min={-95}
              max={-50}
              value={config.settings.rssi_threshold_dbm}
              onChange={(e) =>
                updateSettings({ rssi_threshold_dbm: Number(e.target.value) })
              }
            />
            <strong>{config.settings.rssi_threshold_dbm} dBm</strong>
          </label>
          <button
            className={`btn ${status?.active ? "danger" : "success"}`}
            onClick={toggleMonitor}
            disabled={!config.device}
          >
            {status?.active ? "Arrêter la surveillance" : "Démarrer la surveillance"}
          </button>
          <p className="hint foot">
            Fermer la fenêtre minimise dans la barre système — BTlock reste actif.
          </p>
        </section>

        <section className="card status-card">
          <h2>État</h2>
          <dl>
            <div>
              <dt>Appareil</dt>
              <dd>{config.device?.name ?? "—"}</dd>
            </div>
            <div>
              <dt>Dernière mesure RSSI</dt>
              <dd>
                {status?.last_rssi != null ? `${status.last_rssi} dBm` : "—"}
              </dd>
            </div>
            <div>
              <dt>Série signal faible</dt>
              <dd>{status?.weak_streak ?? 0}</dd>
            </div>
            <div>
              <dt>Événement</dt>
              <dd>{status?.last_event ?? "—"}</dd>
            </div>
          </dl>
        </section>
      </main>
    </div>
  );
}

export default App;
