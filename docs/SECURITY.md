# Sécurité

- **Least privilege** : pas de privilèges admin sans justification explicite.
- **Aucune suppression directe** en V1 : corbeille système uniquement, avec confirmation.
- **Exclusions par défaut** pour dossiers système Windows (voir `dark-pc-core::config`) — **non modifiables**.
- **Exclusions personnelles** (V2.5) : chemins absolus en base ; validation Rust ; impossible d’exclure ou de contourner les zones système.
- Logique centrale : `dark-pc-core::exclusions` (`path_is_skipped`, `effective_from_connection`).
- **Pas de contenu fichier** en base : métadonnées uniquement.
- **Pas de commandes shell dynamiques** pour nettoyer ou supprimer.

Les opérations sensibles seront journalisées dans `operations`.
