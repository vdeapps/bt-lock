# Test RSSI (v1)

## Prérequis

- Linux, BlueZ actif (`systemctl status bluetooth`).
- Utilisateur dans le groupe `bluetooth` si accès refusé.
- Téléphone ou montre : Bluetooth **activé** ; idéalement visible (non connecté en audio seul suffit souvent pour l’advertising LE).

## 1. Trouver l’adresse

Lancer l’app avec **`npm run tauri dev`** (dossier `crates/btlock-app`), pas le navigateur seul.

Dans **BTlock** : **Rechercher** (~8 s), puis **cliquer l’appareil** — appairage et confiance BlueZ (acceptez sur le téléphone si demandé).

Ou en CLI :

```bash
bluetoothctl scan on
# attendre l’apparition de l’appareil, noter Device XX:XX:...
scan off
```

## 2. Sonde RSSI (CLI)

Remplacez l’adresse par la vôtre :

```bash
cd /chemin/vers/BTlock
cargo run -p btlock-bt --example monitor_rssi -- "XX:XX:XX:XX:XX:XX"
```

Éloignez / rapprochez l’appareil : les valeurs doivent **baisser** (ex. -34 proche, -60 à 2 m). Seuil par défaut dans l’app : **-80 dBm**.

## 3. Test verrouillage (app)

1. Sélectionner l’appareil.
2. **Mode** : laisser **Connexion Bluetooth** (défaut) pour une détection fiable **écran éteint** ; ou **Distance RSSI** si vous calibrez le seuil.
3. **Démarrer la surveillance** — l’état affiche **Connecté / Déconnecté** en mode connexion.
4. Éloigner l’appareil (ou couper BT) jusqu’à N lectures « absent » + délai → `loginctl lock-session`.

En mode RSSI, seules les mesures **de moins de 3 secondes** comptent ; une valeur figée à l’écran signifie une mesure expirée (—).

### Realme GT 7 Pro (écran éteint)

- Garder l’appareil **appairé et de confiance** dans BlueZ.
- Désactiver l’optimisation batterie agressive pour Bluetooth / BTlock si la connexion tombe en veille.
- Préférer le mode **Connexion Bluetooth** : la session ACL reste souvent active même écran éteint, contrairement au scan RSSI seul.

Pour un essai sans lock réel, augmentez le délai ou le nombre de lectures faibles.

## 4. Rapporter

Noter : adresse, RSSI proche / loin, comportement au lock. En cas d’erreur BlueZ, coller le message affiché dans l’onglet **État** ou la CLI.

## Calibration Realme GT 7 Pro (suricate-laptop, écran allumé)

| Situation | RSSI observé |
|-----------|----------------|
| À côté du PC | **-34 dBm** |
| ~**2 m** | **-60 dBm** |

Réglage indicatif dans l’app (3 lectures faibles, délai 5 s) :

| Objectif | Seuil RSSI |
|----------|------------|
| Verrouiller vers **2 m** et plus loin | **-62 à -65 dBm** (juste sous -60 à 2 m) |
| Verrouiller seulement **hors bureau / pièce** | **-75 à -80 dBm** (défaut **-80**) |

Pente observée : ~13 dBm/m entre coller et 2 m (orientation et murs peuvent décaler de ±5 dBm).

## Ancienne mesure (autre contexte)

| Situation | RSSI |
|-----------|------|
| Collé | ≈ -39 dBm |
| ~5 m | ≈ -54 dBm |

Si la valeur reste bloquée, **resélectionnez l’appareil** après mise à jour BTlock.
