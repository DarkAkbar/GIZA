# Giza

Assistant desktop de gestion du stockage (Windows en V1).  
Stack : **Tauri 2**, **React**, **TypeScript**, **Tailwind CSS**, **Rust**, **SQLite**.

> Le dépôt / crate Rust peut encore s’appeler `dark-pc` en interne ; le produit utilisateur est **Giza**.

## Prérequis

- [Node.js](https://nodejs.org/) (LTS)
- [Rust (rustup)](https://www.rust-lang.org/tools/install) — après installation, **fermer puis rouvrir** le terminal
- **Build Tools C++** (Visual Studio Build Tools, charge « Développement Desktop en C++ ») — requis pour `link.exe`
- [Prérequis Tauri (Windows + WebView2)](https://tauri.app/start/prerequisites/)

### Erreur `program not found` (cargo)

Rust n’est pas dans le PATH. Réinstaller avec `winget install Rustlang.Rustup` ou vérifier que `%USERPROFILE%\.cargo\bin` est dans le PATH.

### Erreur `link.exe not found`

Installer les [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) avec la charge **MSVC v143** / outils C++.

## Démarrage

```bash
npm install
npm run tauri:dev
```

Sous Cursor/VS Code, ouvrez un **nouveau** terminal après l’installation de Rust (le PATH y inclut `.cargo\bin` via `.vscode/settings.json`).

Commande équivalente manuelle :

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
npm run tauri dev
```

Build frontend seul (sans backend) :

```bash
npm run build
npm run dev
```

Installeur Windows + artefacts de mise à jour :

```powershell
.\scripts\release.ps1
```

Voir [`docs/RELEASE.md`](docs/RELEASE.md) pour GitHub Releases et l’auto-update.

## Structure

| Chemin | Rôle |
|--------|------|
| `src/` | Interface React (pages, layout, types) |
| `src-tauri/` | Shell Tauri, commandes IPC, état applicatif |
| `crates/dark-pc-core/` | Moteur métier et persistance (testable sans UI) |
| `migrations/` | Schéma SQLite versionné |

## Documentation

| Document | Contenu |
|----------|---------|
| [`docs/ETAT_PROJET.md`](docs/ETAT_PROJET.md) | **État complet** : historique des phases, fonctionnalités, IPC, limites |
| [`docs/MVP.md`](docs/MVP.md) | Checklist livraisons V1 / V2 |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Architecture et commandes IPC |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Principes sécurité |
| [`docs/WINDOWS_SETUP.md`](docs/WINDOWS_SETUP.md) | Rust, MSVC, WebView2 |

Principe produit : **analyser → expliquer → agir avec accord explicite**.
