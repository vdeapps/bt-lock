# BTlock — design v1 (Linux)

## Objectif

Verrouiller la session utilisateur lorsqu’un périphérique Bluetooth configuré (téléphone ou montre) s’éloigne, avec paramètres, barre système et exécution **uniquement** via l’application Tauri (pas de mode navigateur).

## Stack

| Couche | Choix | Rôle |
|--------|--------|------|
| OS v1 | Linux | logind + BlueZ |
| Logique | Rust `btlock-engine` | Surveillance, config, lock |
| UI | Tauri 2 + React | WebView système, IPC invoke |
| BT | `bluer` → BlueZ | Scan, pair, trust, RSSI, ACL |
| Session | logind | `loginctl lock-session` par défaut |
| Tray | Tauri tray | App residente (fenêtre masquée à la fermeture) |

## Composants

```text
crates/
  btlock-core/      # LockSettings, ProximityMethod, classify_signal, config JSON
  btlock-bt/        # ActiveLink, scan, associate, read_fresh_rssi, presence_via_link
  btlock-engine/    # engine_start_monitoring, run_monitor, hooks session locked
  btlock-app/       # Tauri commands + React (src/)
  btlock-gui/       # Variante GTK (optionnelle, même engine)
```

## Modes de proximité

| Mode | Config | Verrouillage |
|------|--------|--------------|
| `link_presence` (défaut) | Connexion ACL maintenue | Déconnexion BT après hysteresis |
| `rssi` | Seuil `rssi_threshold_dbm` | Mesure **&lt; 3 s** sous le seuil |

Politique commune : `max_attempts` lectures « faibles » consécutives, puis `delay_before_lock_secs`, puis confirmation avant lock.

## Configuration

Fichier : `~/.config/btlock/config.json`

Champs principaux : `device`, `settings.delay_before_lock_secs`, `settings.max_attempts`, `settings.rssi_threshold_dbm`, `settings.proximity_method`, `settings.lock_command`.

## Sécurité UI

- `devtools: false` (fenêtre Tauri).
- Capability `core:webview:deny-internal-toggle-devtools`.
- Front : pas de mode démo hors Tauri ; raccourcis DevTools / menu contextuel bloqués dans l’app.
- CSP restrictive (scripts locaux + IPC Tauri).

## Risques connus

- RSSI bruité ; mode RSSI ignore les mesures expirées.
- Android / Realme : veille agressive → préférer `link_presence`, appairage + confiance BlueZ.
- Groupe `bluetooth` requis pour BlueZ côté utilisateur.

## Hors v1

- Windows / macOS (autre stack BT et lock).
- Appli compagnon sur le téléphone.
