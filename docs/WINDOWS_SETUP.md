# Configuration Windows (Tauri + Rust)

## Chaîne d’outils

| Outil | Rôle |
|-------|------|
| **Rust (rustup)** | `cargo`, compilateur |
| **MSVC Build Tools** | `link.exe` — **obligatoire** avec la cible `x86_64-pc-windows-msvc` |
| **WebView2** | moteur web embarqué dans l’app |

## Erreur `link.exe not found`

Rust est installé, mais le **linker Microsoft C++** ne l’est pas.

1. Installer [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Cocher **« Développement Desktop en C++ »** (MSVC v143 + Windows SDK)
3. Redémarrer le terminal, puis :

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
npm run tauri dev
```

Installation silencieuse (PowerShell admin ou validation winget) :

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

La première compilation Rust peut prendre **plusieurs minutes**.
