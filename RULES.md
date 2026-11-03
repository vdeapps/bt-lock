# Règles du projet

Ces règles priment sur celles des assistants de développement.

Branche de base : `main`

## Périmètre v1

- **OS** : Linux uniquement (session via logind ; Bluetooth via BlueZ).
- **UI produit** : **Tauri 2** (Rust + WebView système + React). L’UI **ne doit pas** être utilisable dans un navigateur seul ; DevTools désactivés en release.
- **Variante optionnelle** : `btlock-gui` (GTK 4 / Libadwaita), hors produit principal.
- **Appareils** : téléphone ou montre ; proximité via **connexion ACL** (`link_presence`, défaut) ou **seuil RSSI** (mesures fraîches &lt; 3 s).

## Architecture crates

| Crate | Rôle |
|-------|------|
| `btlock-core` | Config JSON, politique de verrouillage, `loginctl` |
| `btlock-bt` | Scan, appairage, RSSI, connexion (BlueZ / `bluer`) |
| `btlock-engine` | Surveillance, persistance, boucle lock |
| `btlock-app` | Binaire Tauri + frontend React |
| `btlock-gui` | Binaire GTK optionnel |

## Documentation

- Besoin métier : `docs/besoin/besoin.md`
- Architecture : `DESIGN.md`
- Build release : `docs/build.md`
- **Guide utilisateur** : `docs/guide-utilisateur.md`
- Installation : `docs/installation.md`
- Référence config : `docs/configuration.md`
- Tests RSSI / calibration : `docs/test-rssi.md`
- Assistants : `AGENTS.md`, `CLAUDE.md`

## Qualité

- `make test`, `make lint`, `make typecheck` (workspace Cargo ; Tauri dans `members`).

## Branches

- Ne jamais merger `staging` dans une autre branche.

## Exceptions

- Aucune pour l'instant.
