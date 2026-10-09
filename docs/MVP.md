# MVP V1

## Livré (squelette)

- Projet Tauri + React + Tailwind
- Navigation (Dashboard, Analyse, Doublons, Gros fichiers, Historique, Paramètres)
- Nettoyage / Organisation (implémentés en V2 — voir `ETAT_PROJET.md`)
- SQLite migration `001_initial.sql`
- Commandes `get_app_info`, `get_volume_stats`
- Dashboard : espace total / utilisé / disponible + barre d'utilisation (volume système)
- **File Scanner** : choix dossier, progression, annulation, persistance `scans` + `files`

- **Gros fichiers** — filtres 100 Mo–5 Go + seuil personnalisé (dernier scan terminé)

- **Doublons** — pipeline taille → SHA-256, groupes, espace récupérable

- **Safe Delete** — sélection, confirmation, corbeille Windows, historique `operations`

## Durcissement MVP (livré)

- Dashboard : potentiel récupérable (doublons), compteur gros fichiers ≥ 100 Mo, raccourcis vers Analyse / Doublons
- Doublons : blocage corbeille si toutes les copies d’un groupe sont sélectionnées

## V2 (livré)

- **Nettoyage** — analyse TEMP / caches navigateur, niveaux SAFE / REVIEW / RISKY, corbeille confirmée
- **Organisation** — propositions par extension à la racine du scan, déplacement confirmé + historique

## V2.5 (en cours)

- **Étape 2 — Settings** : page Paramètres, persistance SQLite, seuil gros fichiers, toggles généraux
- **Étape 3 — Exclusions** : table `user_exclusions`, logique `exclusions/`, scanner + corbeille + organisation
- **Étape 4 — Storage snapshots** : table `storage_snapshots`, `storage_monitor/`, enregistrement au démarrage + IPC
- **Étape 5 — Historique stockage** : `get_storage_history`, section Dashboard + graphique SVG
- **Étape 6 — Recommendation Engine** : `recommendations/`, `get_recommendations`, section Dashboard (règles déterministes, sans IA)
- **Étape 7 — Dashboard intelligent** : centre de pilotage (état stockage, résumé, recommandations, évolution, stats, actions rapides)
- **Étape 8 — Notifications** : alertes Windows locales, anti-spam, basées sur recommandations + fin d'analyse
- **Étape 9 — UX / robustesse** : états chargement / vide / succès / erreur harmonisés (couche présentation)

## Prochaines étapes V2.5

1. Packaging release Windows
