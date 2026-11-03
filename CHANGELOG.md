# Changelog

Format : date — périmètre ; détail concis.

## 2026-11-03 — v0.1.0 (release Linux)

### Application

- Tauri 2 + React : scan BT, association BlueZ, paramètres, surveillance, tray, fenêtre masquée à la fermeture.
- Modes proximité : **connexion ACL** (`link_presence`) et **seuil RSSI** (mesures &lt; 3 s).
- UI hors Tauri bloquée ; DevTools désactivés (config + ACL + front).
- Build release : `.deb`, `.rpm`, AppImage (voir `docs/build.md`).

### Crates Rust

- `btlock-core` : config `~/.config/btlock/config.json`, politique lock, `ProximityMethod`.
- `btlock-bt` : BlueZ via `bluer`, `ActiveLink`, RSSI frais, reconnexion.
- `btlock-engine` : boucle de surveillance partagée Tauri / GTK.
- `btlock-gui` : variante GTK 4 optionnelle.

### Documentation

- Guide utilisateur, installation, configuration, build : voir `docs/README.md`.

### Qualité

- `make test`, `make lint`, `make typecheck` sur le workspace.
