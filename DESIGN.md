# BTlock — design v1 (Linux)

## Objectif

Verrouiller la session utilisateur lorsqu'un périphérique Bluetooth configuré (téléphone ou montre) s'éloigne de la machine, avec configuration, paramètres et présence en barre système.

## Stack

| Couche | Choix | Raison |
|--------|--------|--------|
| OS v1 | Linux | Besoin exprimé ; APIs session/ BT stables via logind et BlueZ. |
| Backend | Rust | Exigence projet ; logique BT, lock, tray. |
| UI | **Tauri 2** | Écosystème le plus répandu pour bureaux modernes « soignés » (HTML/CSS, animations, thèmes). |
| BT | BLE + BlueZ | Téléphones et montres exposent souvent BLE ; mesure RSSI pour proximité. Classic en option si appareil sans LE. |
| Session | logind (D-Bus) | Verrouillage standard sur bureau Linux. |
| Tray | Tauri tray / lib tray | Application residente après fermeture de la fenêtre principale. |

## Composants (cible)

```text
crates/
  btlock-core/     # modèle config, seuils RSSI, politique retry, lock
  btlock-bt/       # scan, pairing léger, suivi RSSI (async)
  btlock-app/      # binaire Tauri + commandes IPC
src-ui/            # frontend Tauri (à créer au scaffold)
```

## Paramètres utilisateur (besoin)

- Appareil BT sélectionné (identifiant stable : adresse / UUID service).
- Délai avant verrouillage après perte de signal ou RSSI sous seuil.
- Nombre de tentatives / hysteresis (éviter locks intempestifs).
- Seuil RSSI (puissance) et éventuellement lissage (moyenne glissante).
- Commande ou mécanisme de lock ( défaut : session courante logind ).

## Risques connus

- RSSI BLE bruité : prévoir hysteresis et délai.
- Montres en mode économie d'énergie : connexion intermittente.
- Permissions : groupe `bluetooth`, PolicyKit si actions sensibles.

## Prochaines étapes implémentation

1. Scaffold Tauri 2 + workspace Cargo.
2. POC scan BLE et lecture RSSI d'un téléphone à proximité.
3. Intégration lock logind + tray.
