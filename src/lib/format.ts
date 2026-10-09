const UNITS = ["o", "Ko", "Mo", "Go", "To"] as const;

export function formatPercent(value: number): string {
  return `${value.toFixed(1)} %`;
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 o";
  const k = 1024;
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), UNITS.length - 1);
  const value = bytes / Math.pow(k, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${UNITS[i]}`;
}

export function formatIsoDate(iso?: string): string {
  if (!iso) return "—";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString("fr-FR", {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

/** Libellé d'axe temps pour le graphique d'historique stockage. */
export function formatStorageChartLabel(iso: string, periodDays: number): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  const now = new Date();
  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  if (periodDays <= 1) {
    return date.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }
  if (sameDay && periodDays <= 7) {
    return `Auj. ${date.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })}`;
  }
  return date.toLocaleDateString("fr-FR", { day: "numeric", month: "short" });
}

export const LARGE_FILE_PRESETS_MB = [
  { label: "100 Mo", mb: 100, bytes: 100 * 1024 * 1024 },
  { label: "500 Mo", mb: 500, bytes: 500 * 1024 * 1024 },
  { label: "1 Go", mb: 1024, bytes: 1024 * 1024 * 1024 },
  { label: "2 Go", mb: 2048, bytes: 2 * 1024 * 1024 * 1024 },
  { label: "5 Go", mb: 5120, bytes: 5 * 1024 * 1024 * 1024 },
] as const;
