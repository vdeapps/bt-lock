# Guide utilisateur BTlock

BTlock **verrouille automatiquement votre session Linux** lorsque votre **téléphone** ou **montre** Bluetooth n’est plus considéré comme « présent » près de l’ordinateur.

---

## 1. Concepts

| Idée | Détail |
|------|--------|
| **Présence** | Détectée via Bluetooth (BlueZ), pas via GPS ni Wi‑Fi. |
| **Surveillance** | Boucle en arrière-plan tant que l’app tourne (y compris fenêtre fermée, icône barre système). |
| **Verrouillage** | Exécute `loginctl lock-session` par défaut → écran de connexion de votre bureau. |
| **Hystérésis** | Plusieurs lectures « absent » + délai avant lock → évite un verrouillage sur un micro-coupure BT. |

Deux **modes de proximité** (Paramètres) :

1. **Connexion Bluetooth** (`link_presence`) — recommandé  
   L’app maintient un lien avec le téléphone appairé. Si le lien tombe durablement → verrouillage.

2. **Distance RSSI** (`rssi`)  
   Utilise la **force du signal** (dBm). Verrou si le signal frais est **sous un seuil** que vous réglez (distance approximative).

---

## 2. Installation

Voir **[installation.md](installation.md)** (.deb, AppImage, binaire).

---

## 3. Premier usage (pas à pas)

### 3.1 Préparer le téléphone

- Bluetooth **activé**.
- Téléphone **déverrouillé** lors du premier appairage si une popup apparaît.
- Sur Android (Realme, etc.) : éviter l’optimisation batterie agressive pour Bluetooth si la connexion coupe en veille.

### 3.2 Associer l’appareil

1. Ouvrir **BTlock**.
2. Cliquer **Rechercher** (scan ~8 secondes).
3. Dans la liste, cliquer **votre appareil** (nom + adresse MAC).
4. Sur le téléphone : **accepter l’appairage** ; sur Linux l’appareil est marqué **de confiance** (trusted) quand c’est possible.
5. Message de confirmation dans l’app.

Si l’appareil n’apparaît pas : rapprochez-le, réveillez l’écran, relancez **Rechercher**.

### 3.3 Choisir le mode de proximité

**Paramètres → Mode de proximité**

| Choix UI | Usage |
|----------|--------|
| **Connexion Bluetooth (écran éteint)** | Vous voulez un lock fiable quand vous **partez avec le téléphone**, sans régler une distance en mètres. |
| **Distance RSSI (seuil dBm)** | Vous voulez verrouiller à une **distance** (ex. au-delà de ~2 m). |

### 3.4 Régler la sensibilité

| Paramètre | Effet |
|-----------|--------|
| **Délai avant verrouillage** | Temps d’attente après la condition « absent » avant le lock (secondes). |
| **Lectures faibles consécutives** | Nombre de cycles (~2 s chacun) en état « absent / signal faible » avant d’entrer dans le délai. |

**Exemple par défaut** : 3 lectures × ~2 s + 5 s de délai ≈ verrouillage ~11 s après une vraie absence (ordre de grandeur).

En mode **RSSI**, régler aussi le **seuil RSSI** (curseur) — voir section 5.

### 3.5 Démarrer la surveillance

1. **Démarrer la surveillance** (bouton vert).
2. Vérifier la carte **État** :
   - **Connexion BT** : Connecté / Déconnecté (mode connexion).
   - **RSSI** : valeur en dBm si une mesure récente existe (&lt; 3 s en mode RSSI).
   - **Événement** : texte explicatif (signal OK, déconnecté, etc.).

### 3.6 Minimiser l’application

- **Fermer la fenêtre** (croix) → l’app **ne quitte pas** ; icône dans la **barre système**.
- Clic sur l’icône tray → rouvrir la fenêtre.
- Menu tray : **Quitter** pour arrêter complètement BTlock.

### 3.7 Arrêter la surveillance

**Arrêter la surveillance** (bouton rouge) — arrête aussi la logique BT active (re-association au prochain démarrage si besoin).

---

## 4. Mode « Connexion Bluetooth »

**Comportement**

- Au démarrage de la surveillance, l’app **connecte** le téléphone appairé (ACL).
- Tant que **Connecté** → pas de verrouillage.
- Si **Déconnecté** plusieurs cycles de suite → compte « faible » → délai → **verrouillage** si toujours absent.
- Une **tentative de reconnexion** est faite avant de compter « absent » (limiter les faux positifs).

**Bon pour**

- Téléphone en poche en quittant le bureau.
- Écran téléphone **éteint** (souvent plus fiable que le RSSI seul).

**Limites**

- Si Android coupe le BT en veille profonde, le lock peut se déclencher alors que vous êtes encore « proche » logiquement — ajuster batterie / rester appairé.

---

## 5. Mode « Distance RSSI »

**Comportement**

- Pas de verrouillage basé uniquement sur « déconnecté ».
- Chaque ~2 s, si la **dernière mesure RSSI a moins de 3 secondes** et est **sous le seuil** → lecture « faible ».
- Mesure **expirée ou absente** → **pas** de verrouillage (évite les valeurs figées).

**Calibrer le seuil**

1. Mode RSSI activé, surveillance **démarrée**.
2. Noter le RSSI **à côté du PC** et **à la distance** où vous voulez verrouiller (ex. 2 m → -60 dBm).
3. Régler le seuil **juste en dessous** de la mesure à cette distance (ex. **-63 dBm** pour lock au-delà de ~2 m si -60 à 2 m).

Tableau indicatif et procédure détaillée : **[test-rssi.md](test-rssi.md)**.

**Affichage**

- Si **RSSI** affiche « — », la mesure est expirée ou indisponible : rapprochez le téléphone ou resélectionnez l’appareil.

---

## 6. Carte « État » (référence)

| Champ | Signification |
|-------|----------------|
| **Appareil** | Nom du téléphone / montre associé. |
| **Connexion BT** | Connecté / Déconnecté (pertinent surtout en mode connexion). |
| **RSSI (&lt; 3 s)** | Dernière mesure récente en dBm. |
| **Série faible** | Compteur de cycles consécutifs « absent / faible » (vers le lock). |
| **Événement** | Dernier message (OK, délai avant lock, erreur BlueZ, etc.). |

---

## 7. Fichier de configuration

Tous les réglages sont persistés. Détail des champs JSON : **[configuration.md](configuration.md)**.

Chemin : `~/.config/btlock/config.json`

---

## 8. Dépannage

| Symptôme | Actions |
|----------|---------|
| Aucun appareil au scan | BT activé sur le téléphone ; `systemctl status bluetooth` ; groupe `bluetooth`. |
| Erreur appairage | Accepter la popup sur le téléphone ; resélectionner après **Rechercher**. |
| Lock trop tôt | Augmenter délai / lectures faibles ; mode RSSI : baisser le seuil (ex. -75 au lieu de -65). |
| Lock trop tard / jamais | Mode connexion : vérifier que le téléphone se déconnecte bien en s’éloignant ; mode RSSI : seuil plus haut (ex. -65). |
| RSSI figé ou « — » | Mode RSSI : mesure expirée — bouger le téléphone ; resélectionner l’appareil ; préférer mode **Connexion** si écran éteint. |
| « Mesure RSSI expirée » | Normal si pas de scan récent ; pas de lock (volontaire). |
| Verrouillage échoue | Tester `loginctl lock-session` dans un terminal ; ajuster `lock_command` dans la config. |
| App « inactive » dans le navigateur | Normal : utiliser **BTlock** installé ou `tauri dev`, pas `localhost:1420` seul. |

---

## 9. Vie privée et sécurité

- BTlock ne communique pas avec un serveur distant pour la fonctionnalité principale.
- Données locales : config, état de surveillance, association BlueZ sur la machine.
- DevTools / inspecteur web **désactivés** dans l’application livrée.
- Le verrouillage dépend de la **sécurité de votre session Linux** (mot de passe après lock).

---

## 10. Aller plus loin

| Document | Sujet |
|----------|--------|
| [installation.md](installation.md) | Installer .deb / AppImage |
| [build.md](build.md) | Compiler depuis les sources |
| [configuration.md](configuration.md) | Référence `config.json` |
| [test-rssi.md](test-rssi.md) | Calibration RSSI, Realme |
| [DESIGN.md](../DESIGN.md) | Architecture technique |
