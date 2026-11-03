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

## Exemple de calibration (suricate-laptop)

| Situation | RSSI observé |
|-----------|----------------|
| Téléphone collé au PC | ≈ **-39 dBm** |
| Téléphone à ~**5 m** | ≈ **-54 dBm** |

Réglage indicatif dans l’app :

- Verrouiller **vers 5 m et au-delà** : seuil **-55 à -58 dBm** (juste sous la mesure à 5 m), 3 lectures faibles, délai 5 s.
- Verrouiller **seulement hors pièce / très loin** : **-70 dBm** ou plus bas (ex. -75).
- Toujours valider en s’éloignant progressivement : le RSSI varie selon murs, orientation et modèle de téléphone.

Si la valeur reste bloquée (ex. toujours -54 dBm), **resélectionnez l’appareil** après mise à jour BTlock : l’app écoute désormais les mises à jour BlueZ en continu (scan LE + événements RSSI).
