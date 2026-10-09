# Publication Giza (Windows)

## Build installateur

1. Fermer `npm run tauri dev`.
2. Clé de signature (déjà générée localement, **privée** dans `src-tauri/giza-updater.key`, ignorée par git) :

```powershell
cd c:\wamp64\www\dark-pc
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = "src-tauri/giza-updater.key"
npm run build:win
```

3. Artefacts :
   - Installeur : `src-tauri\target\release\bundle\nsis\`
   - Mise à jour : `*.nsis.zip` + `.sig` (si `createUpdaterArtifacts` est activé)

## Auto-update (GitHub Releases)

1. Remplacer `YOUR_ORG/giza` dans `src-tauri/tauri.conf.json` → `plugins.updater.endpoints`.
2. Publier `latest.json` (modèle : `docs/release/latest.json`) sur la release GitHub.
3. Signer les bundles avec la **même** clé privée que le build.

## Identifiant application

- Identifiant Tauri : `com.giza.app`
- Base SQLite : `%AppData%\com.giza.app\giza.db`
- Migration automatique depuis `%AppData%\com.darkpc.app\dark-pc.db` au premier lancement.

## Signature code (optionnel)

Pour un installeur signé Authenticode, utiliser un certificat Windows et les options de bundle Tauri / NSIS documentées sur [tauri.app](https://v2.tauri.app/distribute/).
