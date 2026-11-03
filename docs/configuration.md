# Configuration BTlock

## Fichier

Chemin par défaut :

```text
~/.config/btlock/config.json
```

L’application enregistre les changements faits dans l’interface (paramètres, appareil sélectionné). Vous pouvez éditer le JSON à la main **à l’arrêt** de la surveillance si besoin.

## Exemple complet

```json
{
  "settings": {
    "delay_before_lock_secs": 5,
    "max_attempts": 3,
    "rssi_threshold_dbm": -63,
    "lock_command": "loginctl lock-session",
    "proximity_method": "link_presence"
  },
  "device": {
    "address": "AA:BB:CC:DD:EE:FF",
    "name": "Mon téléphone",
    "kind": "phone"
  },
  "monitoring_enabled": false
}
```

## Champs `settings`

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `delay_before_lock_secs` | entier | `5` | Secondes d’attente **après** la décision « absent / signal faible », avant d’exécuter le verrouillage (permet de revenir près du PC). |
| `max_attempts` | entier | `3` | Nombre de cycles de surveillance consécutifs « faibles » (déconnecté ou RSSI sous seuil) avant de lancer le délai de verrouillage. |
| `rssi_threshold_dbm` | entier (négatif) | `-80` | Seuil en dBm en mode **`rssi`** uniquement : verrou si mesure **fraîche** (&lt; 3 s) **strictement sous** ce seuil. |
| `lock_command` | chaîne | `loginctl lock-session` | Commande shell exécutée pour verrouiller (session courante logind). |
| `proximity_method` | enum | `link_presence` | `link_presence` ou `rssi` (voir ci-dessous). |

### `proximity_method`

| Valeur | Comportement | Quand l’utiliser |
|--------|--------------|------------------|
| `link_presence` | Maintient une connexion Bluetooth ; verrouille si le téléphone reste **déconnecté** (reconnexion tentée avant de compter « absent »). | **Recommandé** : écran téléphone éteint, fiabilité maximale sans calibrer la distance. |
| `rssi` | Pas de logique « distance » par connexion seule ; utilise le **seuil RSSI** sur mesures de moins de **3 secondes**. | Verrouiller à une **distance** approximative (ex. ~2 m avec seuil -62 à -65 dBm). |

## Champs `device`

| Champ | Description |
|-------|-------------|
| `address` | Adresse MAC Bluetooth (format `XX:XX:XX:XX:XX:XX`). |
| `name` | Nom affiché (scan BlueZ). |
| `kind` | `phone`, `watch`, ou `unknown` (cosmétique). |

`null` si aucun appareil sélectionné.

## `monitoring_enabled`

État persisté ; l’UI resynchronise au démarrage. En pratique, utilisez les boutons **Démarrer / Arrêter** la surveillance.

## Personnaliser la commande de verrouillage

Défaut adapté à la plupart des bureaux Linux :

```json
"lock_command": "loginctl lock-session"
```

Alternatives possibles (selon votre environnement) :

- `xdg-screensaver lock` (legacy, selon distro)
- Script maison appelant votre gestionnaire de session

La commande est découpée en arguments (premier token = exécutable).

## Réglages conseillés (scénarios)

### Bureau : verrouiller quand je quitte la pièce avec le téléphone

- Mode : **`link_presence`**
- `max_attempts` : 3
- `delay_before_lock_secs` : 5

### Verrouiller vers ~2 m (calibré RSSI, ex. Realme)

- Mode : **`rssi`**
- `rssi_threshold_dbm` : **-62 à -65** (si ~-60 dBm à 2 m)
- Voir [test-rssi.md](test-rssi.md) pour calibrer

### Éviter les faux verrouillages

- Augmenter `max_attempts` (4–5) ou `delay_before_lock_secs` (10–15)
- Mode RSSI : ne pas monter le seuil trop haut (verrouillerait trop tôt)

## Sécurité

- Ne partagez pas `config.json` (contient l’identifiant de votre appareil).
- L’application **ne fonctionne pas** dans un navigateur web seul ; pas de mode démo public.
