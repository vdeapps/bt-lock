# Test RSSI (v1)

## Prérequis

- Linux, BlueZ actif (`systemctl status bluetooth`).
- Utilisateur dans le groupe `bluetooth` si accès refusé.
- Téléphone ou montre : Bluetooth **activé** ; idéalement visible (non connecté en audio seul suffit souvent pour l’advertising LE).

## 1. Trouver l’adresse

Dans l’app **BTlock** : **Rechercher** (~8 s), puis **cliquer l’appareil** — l’app tente l’**appairage et la confiance** BlueZ (acceptez sur le téléphone / montre si une popup apparaît).

Ou en CLI :

```bash
bluetoothctl scan on
# attendre l’apparition de l’appareil, noter Device XX:XX:...
scan off
```

## 2. Sonde RSSI (CLI)

Remplacez l’adresse par la vôtre :

```bash
cd /home/vdeverre/Documents/devels/BTlock
cargo run -p btlock-bt --example monitor_rssi -- "XX:XX:XX:XX:XX:XX"
```

Éloignez / rapprochez l’appareil : les valeurs doivent **baisser** (ex. -55 proche, -85 loin). Seuil par défaut dans l’app : **-70 dBm**.

## 3. Test verrouillage (app)

1. Sélectionner l’appareil, ajuster le seuil RSSI (plus haut = verrouille plus tôt).
2. **Démarrer la surveillance**.
3. Éloigner l’appareil jusqu’à 3 lectures faibles + délai 5 s → `loginctl lock-session`.

Pour un essai sans lock réel, augmentez le délai ou le nombre de lectures faibles.

## 4. Rapporter

Noter : adresse, RSSI proche / loin, comportement au lock. En cas d’erreur BlueZ, coller le message affiché dans l’onglet **État** ou la CLI.
