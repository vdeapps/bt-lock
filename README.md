# BTlock

Verrouillage de session **Linux** quand votre **téléphone** ou **montre** Bluetooth s’éloigne (ou sort du seuil RSSI configuré).

## Documentation

| Public | Lien |
|--------|------|
| **Utiliser et régler l’app** | [docs/guide-utilisateur.md](docs/guide-utilisateur.md) |
| **Installer** (.deb, AppImage) | [docs/installation.md](docs/installation.md) |
| **Fichier config JSON** | [docs/configuration.md](docs/configuration.md) |
| **Build / dev** | [docs/build.md](docs/build.md) |
| **Index complet** | [docs/README.md](docs/README.md) |

## Démarrage rapide (développement)

```bash
cd crates/btlock-app
npm install
npm run tauri dev
```

Build release : voir [docs/build.md](docs/build.md).

Config utilisateur : `~/.config/btlock/config.json`

## Qualité

```bash
make test
make lint
make typecheck
```
