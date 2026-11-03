# Règles du projet

Ces règles priment sur celles des assistants de développement.

Branche de base : main

## Périmètre v1

- **OS** : Linux uniquement (session via logind / D-Bus ; Bluetooth via BlueZ).
- **UI** : **Tauri 2** (backend Rust, frontend web pour interfaces modernes) + icône barre système (tray).
- **Appareils cibles** : téléphone et montre connectée, détection de proximité principalement **BLE** (RSSI) ; Classic à évaluer si un appareil ne expose que du BR/EDR.

## Conventions propres au projet

- Workspace Cargo à la racine ; application Tauri sous `crates/btlock-app/` (ou équivalent une fois scaffoldé).
- Documentation fonctionnelle : `docs/besoin/` ; architecture et choix : `DESIGN.md`.
- Qualité : `make test`, `make lint`, `make typecheck` (aliases vers `cargo`).

## Branches

- Ne jamais merger `staging` dans une autre branche : `staging` est une branche de tests pouvant contenir d'autres branches de travail.

## Commandes particulières

- `make test` / `make lint` / `make typecheck` — contrôles conventionnels orchestrateur.

## Exceptions

- Aucune pour l'instant.
