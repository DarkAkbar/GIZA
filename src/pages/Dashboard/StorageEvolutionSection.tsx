import { useCallback, useEffect, useState } from "react";
import { InlineAlert } from "../../components/InlineAlert";
import { StorageTrendChart } from "../../components/StorageTrendChart";
import { fetchStorageHistory, recordStorageSnapshots } from "../../lib/tauri";
import { formatBytes, formatIsoDate } from "../../lib/format";
import type { StorageHistoryResult } from "../../types/dto";

const PERIODS = [
  { label: "24 h", days: 1 },
  { label: "7 jours", days: 7 },
  { label: "30 jours", days: 30 },
  { label: "90 jours", days: 90 },
] as const;

type StorageEvolutionSectionProps = {
  defaultMountPath?: string;
  /** Si false, n'appelle pas `recordStorageSnapshots` (déjà fait par le Dashboard). */
  recordSnapshotOnLoad?: boolean;
  /** Incrémenter pour recharger depuis le parent (ex. bouton Actualiser). */
  refreshSignal?: number;
};

export function StorageEvolutionSection({
  defaultMountPath,
  recordSnapshotOnLoad = true,
  refreshSignal = 0,
}: StorageEvolutionSectionProps) {
  const [periodDays, setPeriodDays] = useState(7);
  const [mountPath, setMountPath] = useState<string | undefined>(defaultMountPath);
  const [history, setHistory] = useState<StorageHistoryResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (recordSnapshotOnLoad) {
        await recordStorageSnapshots(false);
      }
      const data = await fetchStorageHistory({
        mountPath,
        periodDays,
      });
      setHistory(data);
      setMountPath((prev) => prev ?? data.mountPath);
    } catch (err) {
      setHistory(null);
      setError("Impossible de charger l'historique du stockage.");
    } finally {
      setLoading(false);
    }
  }, [mountPath, periodDays, recordSnapshotOnLoad]);

  useEffect(() => {
    void load();
  }, [load, refreshSignal]);

  const mounts = history?.availableMounts ?? [];
  const showMountSelect = mounts.length > 1;
  const latest = history?.snapshots.length
    ? history.snapshots[history.snapshots.length - 1]
    : null;

  return (
    <section className="mt-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="text-sm font-medium text-[var(--color-text)]">Évolution du stockage</h2>
          <p className="mt-1 text-xs text-[var(--color-text-muted)]">
            Basé sur les snapshots locaux — aucun scan de fichiers.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {showMountSelect ? (
            <select
              value={mountPath ?? history?.mountPath ?? ""}
              onChange={(e) => setMountPath(e.target.value)}
              className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-sm text-[var(--color-text)]"
              aria-label="Volume"
            >
              {mounts.map((m) => (
                <option key={m} value={m}>
                  {m.replace(/\\$/, "")}
                </option>
              ))}
            </select>
          ) : null}
          <select
            value={periodDays}
            onChange={(e) => setPeriodDays(Number(e.target.value))}
            className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-sm text-[var(--color-text)]"
            aria-label="Période"
          >
            {PERIODS.map((p) => (
              <option key={p.days} value={p.days}>
                {p.label}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => void load()}
            disabled={loading}
            className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
          >
            Actualiser
          </button>
        </div>
      </div>

      {loading ? (
        <div className="mt-6 space-y-3 animate-pulse">
          <div className="h-4 w-48 rounded bg-[var(--color-border)]" />
          <div className="h-40 rounded-lg bg-[var(--color-border)]/60" />
        </div>
      ) : error ? (
        <InlineAlert variant="error" className="mt-6" title="Historique indisponible" onRetry={() => void load()}>
          {error}
        </InlineAlert>
      ) : !history?.snapshots.length ? (
        <InlineAlert variant="info" className="mt-6" title="Pas encore assez de données">
          L&apos;évolution s&apos;affichera après plusieurs snapshots (au démarrage, puis au plus toutes les
          15 minutes par volume).
        </InlineAlert>
      ) : (
        <>
          <div className="mt-4 grid gap-3 sm:grid-cols-3 text-sm">
            <div>
              <p className="text-xs text-[var(--color-text-muted)]">Utilisé</p>
              <p className="font-medium text-[var(--color-text)]">
                {formatBytes(latest?.usedBytes ?? 0)}
              </p>
            </div>
            <div>
              <p className="text-xs text-[var(--color-text-muted)]">Libre</p>
              <p className="font-medium text-[var(--color-text)]">
                {formatBytes(latest?.freeBytes ?? 0)}
              </p>
            </div>
            <div>
              <p className="text-xs text-[var(--color-text-muted)]">Total</p>
              <p className="font-medium text-[var(--color-text)]">
                {formatBytes(latest?.totalBytes ?? 0)}
              </p>
            </div>
          </div>
          <p className="mt-2 text-xs text-[var(--color-text-muted)]">
            Dernière mesure : {formatIsoDate(latest?.createdAt)}
          </p>
          <div className="mt-4">
            <StorageTrendChart snapshots={history.snapshots} periodDays={history.periodDays} />
          </div>
        </>
      )}
    </section>
  );
}
