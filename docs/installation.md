# Installation BTlock (Linux)

BTlock v1 cible **Linux** avec **BlueZ** (Bluetooth) et une session graphique verrouillable via **logind** (GNOME, KDE, etc.).

## Prérequis système

| Élément | Vérification |
|---------|----------------|
| Bluetooth | `systemctl status bluetooth` — service actif |
| Droits BT | Utilisateur membre du groupe `bluetooth` si scan/appairage refusé |
| Verrouillage session | `loginctl lock-session` doit fonctionner sur votre bureau |
| Téléphone / montre | Bluetooth activé, appairable |

```bash
# Ajouter l'utilisateur au groupe bluetooth (déconnexion/reconnexion ensuite)
sudo usermod -aG bluetooth "$USER"
```

## Option A — Paquet Debian (.deb)

Après un [build](build.md) ou réception du fichier `.deb` :

```bash
sudo dpkg -i target/release/bundle/deb/BTlock_*_amd64.deb
# ou le chemin de votre .deb
sudo apt-get install -f   # si dépendances manquantes
```

Lanceur : chercher **BTlock** dans le menu applications, ou :

```bash
btlock-app
```

(Le nom exact du binaire installé dépend du paquet ; souvent `btlock-app` ou via le menu `.desktop`.)

## Option B — AppImage

```bash
chmod +x BTlock_*_amd64.AppImage
./BTlock_*_amd64.AppImage
```

Portable : pas d’installation système ; pratique pour tester.

## Option C — Binaire release (sans installateur)

```bash
./target/release/btlock-app
```

Copiez le binaire où vous voulez ; créez un raccourci `.desktop` manuellement si besoin.

## Option D — Développement

Voir [build.md](build.md) section « Développement » (`npm run tauri dev`).

## Variante GTK (optionnelle)

Hors paquet Tauri : compiler `btlock-gui` avec Rust + GTK — voir `crates/btlock-gui/README.md`.

## Désinstallation (.deb)

```bash
sudo dpkg -r btlock-app
# nom du paquet : dpkg -l | grep -i btlock
```

La configuration utilisateur **n’est pas** supprimée : `~/.config/btlock/config.json`.

## Premier lancement

1. Ouvrir BTlock.
2. **Rechercher** un appareil Bluetooth (~8 s).
3. Cliquer votre **téléphone** ou **montre** → accepter l’**appairage** sur l’appareil si demandé.
4. Régler les **paramètres** (voir [configuration.md](configuration.md) et [guide-utilisateur.md](guide-utilisateur.md)).
5. **Démarrer la surveillance**.
6. Fermer la fenêtre : l’app reste dans la **barre système** (tray).

## Dépannage installation

| Problème | Piste |
|----------|--------|
| Pas d’icône tray après install | Paquet build avec `libayatana-appindicator3-dev` ; vérifier extension « AppIndicator » sur GNOME |
| « Bluetooth refused » | Groupe `bluetooth`, service `bluetooth` actif |
| Lock ne fait rien | Tester `loginctl lock-session` en terminal |
| AppImage ne démarre pas | `chmod +x`, libs FUSE si requis par votre distro |
