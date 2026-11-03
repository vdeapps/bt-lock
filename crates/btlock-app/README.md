# BTlock (Tauri 2)

Application principale du projet.

## Lancer

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Voir `../../docs/build.md`.

## Sécurité

- Pas d’usage fonctionnel dans le navigateur (`isTauri()` requis côté React).
- DevTools désactivés en release ; permission Tauri `deny-internal-toggle-devtools`.

## Backend

Commandes IPC dans `src-tauri/src/lib.rs` — délégation à `btlock-engine`.
