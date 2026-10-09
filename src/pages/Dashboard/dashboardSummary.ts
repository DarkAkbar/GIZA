import type { RecommendationDto, VolumeStats } from "../../types/dto";

export type StorageStatusLevel = "normal" | "warning" | "important" | "unknown";

export type StorageStatusSummary = {
  level: StorageStatusLevel;
  label: string;
  message: string;
};

function severityRank(severity: string): number {
  switch (severity) {
    case "important":
      return 3;
    case "warning":
      return 2;
    case "info":
      return 1;
    default:
      return 0;
  }
}

/** Résumé UX uniquement — s'appuie sur les recommandations triées ou, en secours, sur `usagePercent`. */
export function deriveStorageStatus(
  volume: VolumeStats | null,
  recommendations: RecommendationDto[] | null,
  recommendationsFailed: boolean,
): StorageStatusSummary {
  if (recommendations && recommendations.length > 0) {
    const top = recommendations.reduce((best, cur) =>
      severityRank(cur.severity) > severityRank(best.severity) ? cur : best,
    );
    const level: StorageStatusLevel =
      top.severity === "important"
        ? "important"
        : top.severity === "warning"
          ? "warning"
          : "normal";
    const attention =
      level === "important" || level === "warning"
        ? "Votre stockage mérite votre attention."
        : "Votre stockage est dans une situation normale.";
    return {
      level,
      label:
        level === "important"
          ? "Important"
          : level === "warning"
            ? "Attention"
            : "Normal",
      message: attention,
    };
  }

  if (!recommendationsFailed && recommendations) {
    return {
      level: "normal",
      label: "Normal",
      message: "Rien de particulier à signaler pour le moment.",
    };
  }

  if (volume) {
    const p = volume.usagePercent;
    if (p >= 90) {
      return {
        level: "important",
        label: "Stockage presque plein",
        message: "Votre stockage mérite votre attention.",
      };
    }
    if (p >= 85) {
      return {
        level: "warning",
        label: "Attention",
        message: "Surveillez l'utilisation de votre disque.",
      };
    }
    return {
      level: "normal",
      label: "Normal",
      message: "Voici l'état actuel de votre stockage.",
    };
  }

  return {
    level: "unknown",
    label: "—",
    message: "Voici l'état actuel de votre stockage.",
  };
}

export function statusSurfaceClass(level: StorageStatusLevel): string {
  switch (level) {
    case "important":
      return "border-amber-500/50 bg-amber-500/10";
    case "warning":
      return "border-yellow-500/40 bg-yellow-500/5";
    case "normal":
      return "border-emerald-500/30 bg-emerald-500/5";
    default:
      return "border-[var(--color-border)] bg-[var(--color-surface-elevated)]";
  }
}

export function usageBarClass(level: StorageStatusLevel): string {
  switch (level) {
    case "important":
      return "bg-amber-500";
    case "warning":
      return "bg-yellow-500";
    case "normal":
      return "bg-[var(--color-accent)]";
    default:
      return "bg-[var(--color-accent)]";
  }
}
