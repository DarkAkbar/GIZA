# Architecture

```
React (UI) → Tauri IPC → dark-pc-core → SQLite
                ↓
         Storage / File Engine (Rust)
```

- Le frontend n'effectue **aucune** opération système sensible.
- `dark-pc-core` regroupe scanner, doublons, corbeille, requêtes SQL.
- V2 : `cleanup` (temporaires), `organize` (classement à la racine du scan).
- V2.5 : `storage_monitor` (snapshots), `recommendations` (moteur de règles déterministes), Dashboard comme **vue** (orchestration IPC, pas de logique métier dupliquée).

## IPC V1 (squelette)

| Commande | Description |
|----------|-------------|
| `get_app_info` | Version, chemin DB, version de migration |
| `get_volume_stats` | Espace total / utilisé / disponible (volume optionnel, défaut système) |
| `start_scan` | Démarre un scan (un seul à la fois) |
| `cancel_scan` | Demande l'annulation cooperative |
| `get_scan_status` | Snapshot progression / statut |

| `list_large_files` | Fichiers ≥ seuil (dernier scan `completed` par défaut) |
| `start_duplicate_analysis` | Calcul des doublons (thread + annulation) |
| `list_duplicate_groups` | Groupes du scan |
| `get_duplicate_group_details` | Fichiers d'un groupe |

| `move_files_to_trash` | Corbeille système + journal (chemins absolus validés) |
| `list_operations` | Historique local |
| `get_storage_insights` | Synthèse dashboard (scan + doublons + gros fichiers) |
| `analyze_temp_files` | Inventaire temporaires / caches (V2) |
| `get_organize_proposals` | Propositions de classement (V2) |
| `apply_organize_moves` | Déplacements confirmés (V2) |
| `get_settings` | Préférences utilisateur (V2.5) |
| `update_settings` | Mise à jour partielle des préférences |
| `get_exclusions` | Exclusions système + liste utilisateur |
| `add_user_exclusion` | Ajout dossier (validation + normalisation) |
| `remove_user_exclusion` | Suppression par id |
| `reset_user_exclusions` | Vide les exclusions utilisateur |
| `record_storage_snapshots` | Instantanés volumes (`force` optionnel, cooldown 15 min) |
| `get_storage_history` | Historique snapshots pour graphique (`mount_path`, `period_days`, `limit`) |
| `get_recommendations` | Recommandations agrégées (`include_cleanup_scan` optionnel, défaut non) |
| `get_notification_state` | État notifications (activées, entrées anti-spam) |
| `process_notifications` | Envoie les alertes Windows éligibles (max 2, déduplication) |

Événements : `scan-progress`, `scan-finished`, `duplicate-progress`, `duplicate-finished`.
