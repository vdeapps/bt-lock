# Consignes pour les assistants de développement

Toute demande qui modifie, relit ou livre du code dans ce dépôt passe par le skill **orchestrateur**
(`dev:orchestrateur`) : le charger avant toute autre action. Commit manuel : skill `git`.
Question sur le code : réponse directe avec sources `fichier:ligne`.

Règles propres : `RULES.md` (prioritaires).

## Stack BTlock (v1)

- **Rust** workspace : `btlock-core`, `btlock-bt`, `btlock-engine`, `btlock-app` (Tauri), `btlock-gui` (optionnel).
- **UI principale** : Tauri 2 + React — lancer avec `npm run tauri dev` dans `crates/btlock-app`, pas l’UI seule dans le navigateur.
- **Bluetooth** : BlueZ via `bluer` (Linux).
- **Session** : `loginctl lock-session` (configurable dans `~/.config/btlock/config.json`).
- **Qualité** : `make test`, `make lint`, `make typecheck`.

## Fichiers de référence

- `DESIGN.md` — composants et modes de proximité
- `docs/test-rssi.md` — essais terrain
- `docs/build.md` — build release (.deb, AppImage)
