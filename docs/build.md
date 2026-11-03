# Build et développement BTlock

## Vue d’ensemble

| Composant | Rôle |
|-----------|------|
| `crates/btlock-app` | Application **Tauri 2** (React + Rust) — **produit principal** |
| `crates/btlock-engine` | Logique surveillance / config |
| `crates/btlock-bt` | BlueZ (`bluer`) |
| `crates/btlock-core` | Modèle config et politique de lock |
| `crates/btlock-gui` | Variante GTK optionnelle |

---

## Prérequis — développement Tauri

- Rust (stable), `cargo`
- Node.js + npm
- Dépendances [Tauri v2](https://v2.tauri.app/start/prerequisites/) (WebKitGTK, build essentials, etc.)

Sur Debian/Ubuntu, exemple minimal (compléter selon la doc Tauri) :

```bash
sudo apt update
sudo apt install -y build-essential curl wget file libssl-dev \
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
  libwebkit2gtk-4.1-dev
```

## Prérequis — bundle release (tray + .deb)

En plus du dev :

```bash
sudo apt install -y libayatana-appindicator3-dev
```

Sans ce paquet, `tauri build` peut produire le binaire mais **échouer au bundling** (AppIndicator).

---

## Développement (hot reload)

```bash
cd crates/btlock-app
npm install
npm run tauri dev
```

- Lance Vite sur `127.0.0.1:1420` **pour la WebView Tauri uniquement**.
- Ne pas utiliser l’UI dans Firefox/Chrome seul (volontairement bloquée).

Raccourci depuis la racine du dépôt :

```bash
make run
```

---

## Build release

```bash
cd crates/btlock-app
npm install
npm run tauri build
```

Étapes internes :

1. `npm run build` — compile le front React (`crates/btlock-app/dist/`).
2. `cargo build --release -p btlock-app` — binaire Rust optimisé.

### Artefacts (depuis la racine du dépôt)

| Fichier | Description |
|---------|-------------|
| `target/release/btlock-app` | Binaire seul |
| `target/release/bundle/deb/BTlock_*_amd64.deb` | Paquet Debian |
| `target/release/bundle/rpm/BTlock-*.rpm` | Paquet RPM |
| `target/release/bundle/appimage/BTlock_*_amd64.AppImage` | AppImage |

Tailles indicatives : .deb ~5 Mo, AppImage ~80 Mo (runtime embarqué).

### Profil Rust release

Options dans `crates/btlock-app/src-tauri/Cargo.toml` : LTO, `opt-level = 3`, strip.

---

## Build variante GTK (optionnel)

```bash
sudo apt install -y libgtk-4-dev libadwaita-1-dev libgraphene-1.0-dev pkg-config
cargo build -p btlock-gui --release
# binaire : target/release/btlock
```

---

## Qualité (workspace)

Depuis la racine :

```bash
make test        # cargo test --workspace
make lint        # clippy -D warnings
make typecheck   # cargo check --all-targets
make fmt         # cargo fmt --all
```

Members Cargo par défaut : Tauri app + crates Rust (voir `Cargo.toml`).

---

## Structure front Tauri

```text
crates/btlock-app/
  src/              # React (App.tsx, styles)
  src-tauri/        # Rust Tauri (lib.rs, tauri.conf.json, capabilities)
  dist/             # Sortie Vite (build release)
```

IPC : commandes `invoke` → `btlock-engine`.

---

## Configuration build Tauri

Fichier : `crates/btlock-app/src-tauri/tauri.conf.json`

- `devtools: false` sur la fenêtre principale.
- CSP définie sous `app.security`.
- Identifiant : `com.btlock.app`.

Capabilities : `src-tauri/capabilities/default.json` (dont `deny-internal-toggle-devtools`).

---

## Installation après build

Voir **[installation.md](installation.md)**.

---

## Dépannage build

| Erreur | Solution |
|--------|----------|
| `Can't detect any appindicator library` | Installer `libayatana-appindicator3-dev`, relancer `tauri build` |
| Port 1420 occupé | Arrêter un ancien `tauri dev` : `fuser -k 1420/tcp` |
| WebKit / gtk introuvable | Suivre prérequis Tauri pour votre distribution |
| `close_devtools` en release | Ne pas appeler l’API devtools en release ; utiliser `devtools: false` dans la config |
