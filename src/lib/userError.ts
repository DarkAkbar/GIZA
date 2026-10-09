/** Messages utilisateur à partir d'erreurs techniques (sans modifier le backend). */
export function formatUserError(err: unknown): string {
  const raw =
    err instanceof Error ? err.message : typeof err === "string" ? err : String(err ?? "Erreur inconnue");

  const lower = raw.toLowerCase();

  if (
    lower.includes("invoke") ||
    lower.includes("tauri") ||
    lower.includes("__tauri") ||
    lower.includes("failed to fetch")
  ) {
    return "Cette fonctionnalité nécessite l'application desktop Giza.";
  }
  if (lower.includes("scan actif") || lower.includes("already running") || lower.includes("déjà en cours")) {
    return "Une opération est déjà en cours. Attendez la fin ou annulez-la avant de relancer.";
  }
  if (lower.includes("not found") || lower.includes("introuvable") || lower.includes("enoent")) {
    return "Élément introuvable. Le fichier ou le dossier a peut-être été déplacé ou supprimé.";
  }
  if (lower.includes("permission") || lower.includes("accès") || lower.includes("denied") || lower.includes("refus")) {
    return "Accès refusé. Vérifiez les droits ou fermez l'application qui utilise ce fichier.";
  }
  if (lower.includes("cancelled") || lower.includes("annul")) {
    return "Opération annulée.";
  }

  if (raw.length > 220) {
    return `${raw.slice(0, 220)}…`;
  }
  return raw;
}
