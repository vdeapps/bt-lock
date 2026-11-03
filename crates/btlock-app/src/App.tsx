import { useCallback, useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import "./App.css";

type DeviceKind = "phone" | "watch" | "unknown";

type ProximityMethod = "link_presence" | "rssi";

interface LockSettings {
  delay_before_lock_secs: number;
  max_attempts: number;
  rssi_threshold_dbm: number;
  lock_command: string;
  proximity_method?: ProximityMethod;
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
  link_connected?: boolean | null;
  weak_streak: number;
  last_event: string;
}

const kindLabel: Record<DeviceKind, string> = {
  phone: "Téléphone",
  watch: "Montre",
  unknown: "Autre",
};

function StandaloneBlocked() {
  return (
    <div className="shell blocked">
      <h1>BTlock</h1>
      <p>
        Cette interface ne fonctionne que dans l’application de bureau BTlock (Tauri).
      </p>
      <p className="hint">
        Lancez <code>npm run tauri dev</code> ou le binaire installé — l’ouvrir dans un
        navigateur n’est pas pris en charge.
      </p>
    </div>
  );
}

function BtlockShell() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [devices, setDevices] = useState<BtDevice[]>([]);
  const [status, setStatus] = useState<MonitorStatus | null>(null);
  const [scanning, setScanning] = useState(false);
  const [message, setMessage] = useState<string>("");

  useEffect(() => {
    const blockDevToolsShortcut = (event: KeyboardEvent) => {
      const key = event.key.toLowerCase();
      if (
        key === "f12" ||
        (event.ctrlKey && event.shiftKey && (key === "i" || key === "j" || key === "c"))
      ) {
        event.preventDefault();
        event.stopPropagation();
      }
    };
    const blockContextMenu = (event: MouseEvent) => {
      event.preventDefault();
    };
    window.addEventListener("keydown", blockDevToolsShortcut, true);
    document.addEventListener("contextmenu", blockContextMenu);
    return () => {
      window.removeEventListener("keydown", blockDevToolsShortcut, true);
      document.removeEventListener("contextmenu", blockContextMenu);
    };
  }, []);

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  const refreshStatus = useCallback(async () => {
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
            Mode de proximité
            <select
              value={config.settings.proximity_method ?? "link_presence"}
              onChange={(e) =>
                updateSettings({
                  proximity_method: e.target.value as ProximityMethod,
                })
              }
            >
              <option value="link_presence">
                Connexion Bluetooth (fiable, écran éteint)
              </option>
              <option value="rssi">Distance RSSI (mesures fraîches)</option>
            </select>
          </label>
          {(config.settings.proximity_method ?? "link_presence") ===
          "link_presence" ? (
            <p className="hint">
              Ce mode verrouille quand le téléphone se <strong>déconnecte</strong> du
              PC, pas à une distance en mètres. Pour verrouiller vers 2&nbsp;m (ex.
              −60&nbsp;dBm chez vous), choisissez <strong>Distance RSSI</strong>.
            </p>
          ) : (
            <p className="hint">
              Verrouillage si la mesure fraîche (&lt; 3&nbsp;s) est{" "}
              <strong>sous le seuil</strong>. À 2&nbsp;m vous aviez −60&nbsp;dBm :
              réglez entre <strong>−62 et −65&nbsp;dBm</strong> (juste en dessous de −60
              à 2&nbsp;m).
            </p>
          )}
          {(config.settings.proximity_method ?? "link_presence") === "rssi" && (
            <label>
              Seuil RSSI (verrouiller si mesure sous ce dBm)
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
          )}
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
              <dt>Connexion BT</dt>
              <dd>
                {status?.link_connected == null
                  ? "—"
                  : status.link_connected
                    ? "Connecté"
                    : "Déconnecté"}
              </dd>
            </div>
            <div>
              <dt>Dernière mesure RSSI (&lt; 3 s)</dt>
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

function App() {
  if (!isTauri()) {
    return <StandaloneBlocked />;
  }
  return <BtlockShell />;
}

export default App;
