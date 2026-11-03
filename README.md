# BTlock

Verrouillage de session Linux quand un téléphone ou une montre Bluetooth s’éloigne.

## Prérequis

- Linux, BlueZ, Rust, Node.js npm
- Tauri deps : https://tauri.app/start/prerequisites/

## Développement

```bash
cd crates/btlock-app
npm install
npm run tauri dev
```

Qualité (racine du dépôt) :

```bash
make test
make lint
make typecheck
```

## Configuration

Fichier : `~/.config/btlock/config.json`

Guide test RSSI réel : `docs/test-rssi.md`
