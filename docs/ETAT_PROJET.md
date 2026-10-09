# Dark PC — État du projet et historique d’évolution

**Version applicative :** `0.1.0`  
**Dernière mise à jour document :** septembre 2025  
**Plateforme cible V1 :** Windows (desktop Tauri)

---

## 1. Vision produit

**Dark PC** est un assistant desktop de **gestion du stockage**. Il aide à :

1. **Analyser** l’espace disque et l’arborescence indexée  
2. **Expliquer** où part l’espace (gros fichiers, doublons, temporaires, etc.)  
3. **Agir uniquement avec l’accord explicite de l’utilisateur**

Principes non négociables :

| Principe | Mise en œuvre actuelle |
|----------|-------------------------|
| Pas de suppression définitive automatique | Corbeille Windows (`trash`), modal de confirmation |
| Pas de shell / commandes dynamiques | Opérations via API Rust (`std::fs`, `trash`) |
| Métadonnées en base, pas le contenu | Table `files` : chemins, tailles, dates, hash optionnel |
| Zones sensibles exclues | Liste d’exclusions (`config::default_scan_exclusions`) |
| Journal local | Table `operations` + page Historique |

---

## 2. Stack technique

| Couche | Technologies |
|--------|----------------|
| UI | React 19, TypeScript, Tailwind CSS 4, React Router 7 |
| Shell desktop | Tauri 2, plugins `dialog`, `opener`, `fs` |
| Métier | Crate Rust `dark-pc-core` (testable sans Tauri) |
| Persistance | SQLite (`rusqlite`, migration `001_initial.sql`) |
| Système | `sysinfo` (volumes), `walkdir` (scan), `sha2` (doublons), `trash` (corbeille) |

**Emplacement de la base :** `%AppData%\...\dark-pc.db` (répertoire données Tauri).

---

## 3. Chronologie du développement (phases)

Le projet a avancé **par modules validés**, du squelette jusqu’à la V2.

### Phase initiale — Squelette (étapes 6–7)

- Workspace Cargo : `src-tauri` + `crates/dark-pc-core`
- Navigation : Dashboard, Analyse, Doublons, Gros fichiers, Nettoyage, Organisation, Historique, Paramètres
- Migration SQLite `migrations/001_initial.sql`
- Commande `get_app_info`
- Pages Nettoyage / Organisation en **placeholder** (remplacées en V2)

### Phase A — (intégrée au squelette)

- Structure UI, routing, thème sombre Tailwind

### Phase B — Disque + Dashboard

- Module `disk/` (`sysinfo`)
- IPC `get_volume_stats`
- Dashboard : barre d’utilisation, espace total / utilisé / disponible (volume système, ex. `C:`)

### Phase C — File Scanner

- `scanner/engine.rs` : parcours `walkdir`, exclusions, lots de 256 fichiers, progression, annulation coopérative
- Persistance `scans` + `files` via `db/scan_repo.rs`
- IPC : `start_scan`, `cancel_scan`, `get_scan_status`
- Événements Tauri : `scan-progress`, `scan-finished`
- Plugin dialog : choix de dossier
- Page **Analyse**

### Phase D — Gros fichiers

- `large_files/` : requêtes SQL sur le **dernier scan `completed`**
- IPC `list_large_files`
- Page **Gros fichiers** : presets 100 Mo → 5 Go + seuil personnalisé
- **Correctif post-livraison :** sérialisation JSON `camelCase` (`matchCount`, `sizeBytes`, etc.) pour éviter crash UI

### Phase E — Doublons

- Pipeline **taille → SHA-256** (`duplicates/`, `duplicate_repo`)
- IPC : `start_duplicate_analysis`, `cancel_duplicate_analysis`, `get_duplicate_status`, `list_duplicate_groups`, `get_duplicate_group_details`
- Événements : `duplicate-progress`, `duplicate-finished`
- Page **Doublons** : groupes dépliables, espace récupérable par groupe

### Phase F — Safe Delete + Historique

- `safe_delete/` : validation chemins, corbeille Windows, retrait des entrées indexées réussies
- `operations_repo` : journal `operations`
- IPC : `move_files_to_trash`, `list_operations`
- UI : `ConfirmDialog`, sélection + corbeille sur Doublons, page **Historique**
- Garde-fou métier : conserver au moins une copie par groupe de doublons (message UI)

### Durcissement MVP (post-V1)

- Module `dashboard/` + IPC `get_storage_insights`
- Dashboard enrichi : potentiel récupérable (doublons), compteur gros fichiers ≥ 100 Mo, résumé dernier scan, liens rapides
- **Blocage corbeille** si toutes les copies d’un groupe de doublons sont sélectionnées

### V2 — Nettoyage + Organisation

- **`cleanup/`** + IPC `analyze_temp_files`  
  - Parcours TEMP, `%LOCALAPPDATA%\Temp`, caches Chrome/Edge, `%WINDIR%\Temp`  
  - Niveaux **SAFE / REVIEW / RISKY**  
  - Page **Nettoyage** : filtres, « Sélectionner SAFE », corbeille confirmée  

- **`organize/`** + IPC `get_organize_proposals`, `apply_organize_moves`  
  - Classement par extension des fichiers **à la racine** du dossier scanné  
  - Dossiers cibles : Images, Videos, Documents, Musique, Archives, Installateurs, Autres  
  - Prévisualisation, conflits (destination existante), déplacement + journal `organize`  

- Sidebar : retrait des badges « V2 » sur Nettoyage / Organisation

### Environnement de développement (problèmes résolus)

- `cargo` absent du PATH → rustup, `.vscode/settings.json`, `scripts/tauri-dev.ps1`
- `link.exe` manquant → Visual Studio Build Tools 2022 (charge C++)
- Handler Tauri : `generate_handler![...]`
- Point d’entrée : `main.rs` → `dark_pc_lib::run()`
- État partagé : `Arc<Mutex<...>>` pour scans et analyse doublons en arrière-plan

---

## 4. État fonctionnel par écran

| Écran | Route | Statut | Description |
|-------|--------|--------|-------------|
| Dashboard | `/dashboard` | **Complet (V2.5)** | Centre de pilotage + états UX (chargement / vide / erreur par section) |
| Analyse | `/analyze` | **Complet** | Scan dossier, progression, annulation |
| Doublons | `/duplicates` | **Complet** | Détection, groupes, corbeille sélective, garde-fous |
| Gros fichiers | `/large-files` | **Complet** | Liste triée par taille (dernier scan) |
| Nettoyage | `/cleanup` | **Complet (V2)** | Temporaires / caches, risques, corbeille |
| Organisation | `/organize` | **Complet (V2)** | Propositions + déplacements confirmés |
| Historique | `/history` | **Complet** | Opérations `trash`, `organize`, etc. |
| Paramètres | `/settings` | **Complet (V2.5)** | Seuil, toggles, exclusions système + personnelles |

---

## 5. Commandes IPC (inventaire complet)

### Synchrones (invoke)

| Commande | Module Rust | Rôle |
|----------|-------------|------|
| `get_app_info` | `commands/app` | Nom, version, chemin DB, migration, plateforme |
| `get_volume_stats` | `commands/disk` | Stats volume (mount optionnel) |
| `get_storage_insights` | `commands/dashboard` | Synthèse pour dashboard |
| `start_scan` | `commands/scan` | Démarre un scan (1 actif max) |
| `cancel_scan` | `commands/scan` | Annulation coopérative |
| `get_scan_status` | `commands/scan` | Snapshot scan en cours / idle |
| `list_large_files` | `commands/large_files` | Gros fichiers (seuil, scan optionnel) |
| `start_duplicate_analysis` | `commands/duplicates` | Thread analyse doublons |
| `cancel_duplicate_analysis` | `commands/duplicates` | Annulation analyse |
| `get_duplicate_status` | `commands/duplicates` | Snapshot analyse |
| `list_duplicate_groups` | `commands/duplicates` | Liste groupes |
| `get_duplicate_group_details` | `commands/duplicates` | Membres d’un groupe |
| `move_files_to_trash` | `commands/safe_delete` | Corbeille + journal + MAJ index |
| `list_operations` | `commands/safe_delete` | Historique |
| `analyze_temp_files` | `commands/cleanup` | Inventaire temporaires (V2) |
| `get_organize_proposals` | `commands/organize` | Propositions classement (V2) |
| `apply_organize_moves` | `commands/organize` | Déplacements (V2) |
| `get_settings` | `commands/settings` | Préférences (V2.5) |
| `update_settings` | `commands/settings` | Enregistrement préférences |
| `get_exclusions` | `commands/exclusions` | Vue exclusions système + utilisateur |
| `add_user_exclusion` | `commands/exclusions` | Ajout dossier exclu |
| `remove_user_exclusion` | `commands/exclusions` | Suppression |
| `reset_user_exclusions` | `commands/exclusions` | Réinitialisation liste utilisateur |
| `record_storage_snapshots` | `commands/storage` | Snapshots volumes (cooldown / force) |
| `get_storage_history` | `commands/storage` | Historique snapshots pour graphique |
| `get_recommendations` | `commands/recommendations` | Recommandations calculées à la volée (V2.5 étape 6) |
| `get_notification_state` | `commands/notifications` | Préférence + état anti-spam (V2.5 étape 8) |
| `process_notifications` | `commands/notifications` | Envoi alertes Windows (recommandations warning/important) |

### Événements (listen)

| Événement | Payload (résumé) |
|-----------|------------------|
| `scan-progress` | `scanId`, `filesSeen`, `bytesTotal`, `currentPath`, `status` |
| `scan-finished` | `scanId`, `status`, totaux, `errorMessage` |
| `duplicate-progress` | `scanId`, candidats hashés, groupes, octets récupérables |
| `duplicate-finished` | Résultat final analyse doublons |

---

## 6. Modules Rust (`dark-pc-core`)

| Dossier / fichier | Responsabilité |
|-------------------|----------------|
| `config/` | Exclusions scan par défaut, clés settings |
| `db/` | SQLite, migrations, `scan_repo`, `duplicate_repo`, `operations_repo` |
| `disk/` | Statistiques volumes |
| `scanner/` | Moteur de scan + exclusions |
| `large_files/` | Requêtes gros fichiers |
| `duplicates/` | Détection doublons + moteur hash |
| `safe_delete/` | Corbeille, validation chemins |
| `dashboard/` | Agrégats pour dashboard |
| `settings/` | Chargement / validation préférences SQLite |
| `exclusions/` | Exclusions effectives, normalisation, protection système |
| `db/settings_repo` | Accès clé-valeur `settings` |
| `db/exclusions_repo` | Table `user_exclusions` |
| `storage_monitor/` | Enregistrement instantanés disque |
| `notifications/` | Planification alertes locales + déduplication (settings JSON) |
| `recommendations/` | Moteur de règles déterministes (étape 6) |
| `db/storage_snapshot_repo` | Table `storage_snapshots` |
| `cleanup/` | Analyse temporaires (V2) |
| `organize/` | Propositions et déplacements (V2) |
| `models/` | Types partagés (`RiskLevel`, métadonnées fichier, etc.) |

**Tests unitaires Rust :** 13 tests (`cargo test -p dark-pc-core`) — disque, scan, doublons, gros fichiers, safe_delete, dashboard, cleanup, organize, migrations.

---

## 7. Schéma de données (SQLite v1)

Migrations **`001`–`003`** (`user_exclusions`, `storage_snapshots`) — version schéma **3**.

| Table | Usage |
|-------|--------|
| `settings` | Clés/valeurs (prévu, peu utilisé côté UI) |
| `scans` | Sessions d’analyse (statut, racine, compteurs, dates) |
| `files` | Fichiers indexés par scan (métadonnées, `content_hash` optionnel) |
| `duplicate_groups` | Groupes de doublons par scan |
| `duplicate_group_members` | Liaison groupe ↔ fichier |
| `operations` | Journal : `trash`, `move`, `organize`, `scan`, `restore_note` |
| `schema_migrations` | Versionnement schéma |

**Pas de migration v2 SQL** : V2 réutilise le schéma existant (`operations.type = 'organize'`, etc.).

---

## 8. Exclusions et sécurité

Dossiers **jamais** ciblés par scan / corbeille / déplacement (segments de chemin) :

- `System Volume Information`
- `$Recycle.Bin`
- `Windows`
- `Program Files`, `Program Files (x86)`
- `ProgramData`

Voir `docs/SECURITY.md` et `crates/dark-pc-core/src/config/mod.rs`.

---

## 9. Limites connues et hors périmètre

| Sujet | État |
|-------|------|
| Paramètres UI (exclusions custom, seuils persistés) | Non implémenté |
| Suppression définitive | Volontairement absente |
| Restauration depuis l’app | Non — via Corbeille Windows |
| Organisation récursive (sous-dossiers entiers) | Non — uniquement fichiers à la **racine** du scan |
| Nettoyage : tous les caches / apps | Partiel — TEMP + Chrome/Edge + Temp Windows |
| Surveillance continue du disque | Non (V3+) |
| Moteur de recommandations / IA | Non (V3+) |
| macOS / Linux | Non prévu en V1 |
| ErrorBoundary React global | Non |
| Tests E2E automatisés | Non |
| Installeur / release signée | Non fait (`npm run tauri build` à valider) |

---

## 10. Comment lancer et vérifier

```powershell
npm install
npm run tauri:dev
```

Ou :

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
npm run tauri dev
```

Vérifications développeur :

```powershell
cargo test -p dark-pc-core
cargo check -p dark-pc
npm run build
```

Documentation complémentaire :

- `README.md` — démarrage rapide  
- `docs/ARCHITECTURE.md` — vue IPC / architecture  
- `docs/MVP.md` — checklist livraisons V1/V2  
- `docs/WINDOWS_SETUP.md` — Rust, MSVC, WebView2  
- `docs/SECURITY.md` — principes sécurité  

---

## 11. Prochaines étapes suggérées

1. **Release Windows** — build Tauri, installeur, smoke tests sur machine propre  
2. **Paramètres** — édition exclusions / seuils via table `settings`  
3. **V3+** — surveillance, recommandations, nettoyage étendu, polish UX (ErrorBoundary, refresh dashboard)  

---

## 12. Résumé exécutif

Dark PC **0.1.0** est une application Tauri **fonctionnelle** sur Windows : scan indexé, dashboard, gros fichiers, doublons exacts, corbeille sécurisée, historique, nettoyage de temporaires classés par risque, et organisation simple des téléchargements « en vrac ». Le développement a suivi un **fil rouge produit** (analyser → expliquer → agir avec consentement) et une **architecture propre** (UI ↔ IPC ↔ `dark-pc-core` ↔ SQLite). Il reste surtout du **polish**, des **réglages utilisateur**, et un **packaging release** avant diffusion large.
