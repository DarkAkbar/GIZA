import { formatBytes, formatPercent } from "../../lib/format";
import type { VolumeStats } from "../../types/dto";
import { type StorageStatusLevel, usageBarClass } from "./dashboardSummary";

type StorageOverviewCardProps = {
  volume: VolumeStats | null;
  loading: boolean;
  error: string | null;
  statusLevel: StorageStatusLevel;
  volumeLabel: string;
};

export function StorageOverviewCard({
  volume,
  loading,
  error,
  statusLevel,
  volumeLabel,
}: StorageOverviewCardProps) {
  if (loading) {
    return (
      <section
        className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5"
        aria-busy="true"
      >
        <div className="animate-pulse space-y-4">
          <div className="h-4 w-32 rounded bg-[var(--color-border)]" />
          <div className="h-10 w-24 rounded bg-[var(--color-border)]" />
          <div className="h-2 rounded-full bg-[var(--color-border)]" />
          <div className="grid gap-3 sm:grid-cols-4">
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="h-14 rounded-lg bg-[var(--color-border)]/60" />
            ))}
          </div>
        </div>
      </section>
    );
  }

  if (error || !volume) {
    return (
      <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
        <p className="text-sm text-red-400">{error ?? "Impossible de lire le volume système."}</p>
      </section>
    );
  }

  const pct = Math.min(100, Math.max(0, volume.usagePercent));

  return (
    <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
      <p className="text-xs uppercase tracking-wide text-[var(--color-text-muted)]">{volumeLabel}</p>
      <div className="mt-3 flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-3xl font-semibold tabular-nums text-[var(--color-text)]">
            {formatPercent(volume.usagePercent)}
          </p>
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">Utilisation du volume</p>
        </div>
        <p className="text-sm text-[var(--color-text-muted)]">
          {formatBytes(volume.usedBytes)} utilisés · {formatBytes(volume.availableBytes)} libres
        </p>
      </div>
      <div
        className="mt-4 h-2.5 overflow-hidden rounded-full bg-[var(--color-border)]"
        role="progressbar"
        aria-valuenow={pct}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={`Utilisation du stockage ${formatPercent(volume.usagePercent)}`}
      >
        <div
          className={`h-full rounded-full transition-all duration-500 ${usageBarClass(statusLevel)}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <dl className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Metric label="Utilisé" value={formatBytes(volume.usedBytes)} />
        <Metric label="Libre" value={formatBytes(volume.availableBytes)} />
        <Metric label="Total" value={formatBytes(volume.totalBytes)} />
        <Metric label="Utilisation" value={formatPercent(volume.usagePercent)} />
      </dl>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3">
      <dt className="text-xs text-[var(--color-text-muted)]">{label}</dt>
      <dd className="mt-1 text-lg font-semibold tabular-nums text-[var(--color-text)]">{value}</dd>
    </div>
  );
}
