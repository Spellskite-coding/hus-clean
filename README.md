# Hus-Clean

**Hus-Clean** est un outil léger et sécurisé pour nettoyer les fichiers temporaires et les cookies inutiles (publicités, trackers) sur Windows. Développé en C++/Qt, il est conçu pour être simple, efficace et respectueux de la vie privée.

---

## 📌 Fonctionnalités

- **Nettoyage des fichiers temporaires** (`%TEMP%`, `%LOCALAPPDATA%\Temp`).
- **Suppression des cookies indésirables** (Chrome, Firefox, Brave, Edge, Opera, Vivaldi).
- **Interface intuitive** avec confirmation avant toute suppression.
- **Pas de droits administrateur requis** (pas de demande UAC).
- **Transparence totale** : Aucun fichier n'est supprimé sans confirmation explicite.

---

## 📥 Installation

### Prérequis
- Windows 7/10/11.
- [Qt 5.15+](https://www.qt.io/) (pour compiler depuis le code source).

### Téléchargement
**Version précompilée** :
  Clone ce dépôt ou télécharge le .zip en cliquant sur "Code" → "Download ZIP" ou télécharge la dernière version depuis les [releases GitHub](https://github.com/Spellskite-coding/hus-clean/releases/).


## 🚀 Utilisation :

Lance Hus-Clean (HusClean.exe).
Clique sur "Scanner" pour lister les fichiers temporaires et cookies.
Coche les cases des fichiers à supprimer.
Clique sur "Nettoyer" et confirme.
Vérifie : Les fichiers sélectionnés sont supprimés (sauf s'ils sont verrouillés par un navigateur ouvert).


## ⚠️ Attention :

Ferme tes navigateurs avant de nettoyer les cookies.
Hus-Clean ne supprime que les fichiers listés et confirmés.


## 🛠️ Compilation :

Avec Qt Creator

Ouvre le projet hus-clean.pro.
Compile (Build > Build Project).
L'exécutable sera généré dans build/release/.
Intégration du manifest

## Le projet inclut un manifest Windows (HusClean.exe.manifest) pour :

Éviter les demandes UAC et décrire clairement l'outil aux antivirus.

© 2026 Spellskite-coding (https://github.com/Spellskite-coding) & Thomas Scholl (https://github.com/Skoll1470) & Marwane Toury (https://github.com/MarwaneToury).
