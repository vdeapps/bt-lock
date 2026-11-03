# btlock-engine

Logique applicative BTlock sans UI : configuration, association, boucle de surveillance, verrouillage session.

Consommé par :

- `btlock-app` (Tauri)
- `btlock-gui` (GTK, optionnel)

## API principale

- `new_engine()` → `SharedEngine`
- `engine_scan`, `engine_select_device`, `engine_start_monitoring`, `engine_stop_monitoring`
- `engine_get_config`, `engine_save_config`, `engine_get_status`
- `engine_set_session_locked_hook` (callback après lock, ex. événement Tauri)

## Tests

```bash
cargo test -p btlock-engine
cargo clippy -p btlock-engine -- -D warnings
```
