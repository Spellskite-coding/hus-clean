# Hus-Clean

**Hus-Clean** est un outil léger et sécurisé pour nettoyer les fichiers temporaires et les cookies/cache inutiles (publicités, trackers) sur Windows. Cette version est une réécriture complète en **Rust**, conçue pour être plus robuste, plus légère et plus respectueuse de la vie privée que la version C++/Qt d'origine.

---

## 📌 Fonctionnalités

- **Nettoyage des fichiers temporaires** (`%TEMP%`, `%LOCALAPPDATA%\Temp`), scan **récursif** (sous-dossiers inclus).
- **Suppression des cookies indésirables** (Chrome, Firefox, Brave, Edge, Opera, Vivaldi), avec détection de **plusieurs profils** par navigateur.
- **Nettoyage du cache des navigateurs**, en plus des cookies.
- **Envoi à la corbeille** plutôt que suppression définitive : possibilité d'annuler.
- **Sélection individuelle des fichiers** avant nettoyage (case à cocher par élément).
- **Interface intuitive** avec confirmation avant toute suppression.
- **Pas de droits administrateur requis** (pas de demande UAC).
- **Aucune connexion réseau** : zéro télémétrie, par construction.
- **Transparence totale** : aucun fichier n'est supprimé sans confirmation explicite.

---

## 📥 Installation

### Prérequis
- Windows 10/11.

### Téléchargement
Récupère simplement **`hus-clean.exe`** à la racine de ce dépôt et place-le où tu veux sur ton ordinateur. C'est un exécutable unique et autonome : pas d'archive à décompresser, pas de dossier de `.dll` à côté.

Voir **`Procédure Hus-Clean.pdf`** pour un guide pas à pas illustré (téléchargement, déblocage Windows, utilisation).

---

## 🚀 Utilisation

1. Lance `hus-clean.exe`.
2. Clique sur **"Scanner"** et choisis ce que tu veux analyser (fichiers temporaires / cookies / cache).
3. Décoche les éléments que tu veux conserver.
4. Clique sur **"Nettoyer"** et confirme.
5. Les éléments sélectionnés sont envoyés à la corbeille (sauf ceux verrouillés par un navigateur ouvert ou un fichier système en cours d'utilisation, clairement signalés).

---

## ⚠️ Attention

- Ferme tes navigateurs avant de nettoyer leurs cookies/cache.
- Le logiciel n'étant pas signé numériquement (signature de code payante), Windows peut afficher un avertissement au premier lancement (blocage "provient d'un autre ordinateur" + SmartScreen) : voir la procédure PDF pour les débloquer, c'est normal pour tout logiciel gratuit non signé.

---

## 🛠️ Compilation depuis les sources

Le code source se trouve dans le dossier [`src/`](src/). C'est un projet Rust (workspace Cargo) :

- `src/crates/hus-clean-core` : logique de scan et de nettoyage, sans dépendance graphique, testable indépendamment (`cargo test`).
- `src/crates/hus-clean-gui` : interface graphique ([egui](https://github.com/emilk/egui)/eframe).
- `src/docker` : Dockerfiles pour reproduire l'environnement de cross-compilation (Linux → Windows, via `mingw-w64` et `cargo-xwin`/MSVC) et de test.

Compilation (depuis `src/`, avec le toolchain Rust + cible `x86_64-pc-windows-msvc` installée, voir `src/docker/Dockerfile`) :

```
cargo xwin build --release --target x86_64-pc-windows-msvc
```

---

## Sécurité

- Aucun `unsafe` dans le code du projet (vérifié via `cargo-geiger`).
- Dépendances auditées via `cargo-audit` (base de vulnérabilités RustSec).
- Analyse statique via `cargo clippy`.
- Le binaire ne lie que des DLL système Windows standard, aucune dépendance externe à installer.

---

© 2026 Spellskite-coding (https://github.com/Spellskite-coding) & Marwane Toury (https://github.com/MarwaneToury).
