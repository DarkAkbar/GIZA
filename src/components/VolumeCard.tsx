import { formatBytes, formatPercent } from "../lib/format";
import type { VolumeStats } from "../types/dto";

type VolumeCardProps = {
  volume: VolumeStats;
  selected?: boolean;
  onSelect?: () => void;
  actionLabel?: string;
  compact?: boolean;
};

export function VolumeCard({
  volume,
  selected,
  onSelect,
  actionLabel = "Analyser",
  compact,
}: VolumeCardProps) {
  const mountLabel = volume.mountPath.replace(/\\$/, "");
  const pct = Math.min(100, Math.max(0, volume.usagePercent));

  return (
    <article
      className={[
        "dp-card flex flex-col gap-3 transition-colors",
        selected ? "ring-2 ring-[var(--color-accent)]/50" : "",
        compact ? "p-3" : "p-4",
      ].join(" ")}
    >
      <div className="flex items-start justify-between gap-2">
        <div>
          <p className="text-xs uppercase tracking-wide text-[var(--color-text-muted)]">Volume</p>
          <p className="text-lg font-semibold text-[var(--color-text)]">{mountLabel}</p>
        </div>
        <p className="text-sm font-medium tabular-nums text-[var(--color-text)]">
          {formatPercent(volume.usagePercent)}
        </p>
      </div>
      <div
        className="h-2 overflow-hidden rounded-full bg-[var(--color-border)]"
        role="progressbar"
        aria-valuenow={pct}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          className="h-full rounded-full bg-gradient-to-r from-[var(--color-accent)] to-amber-400/90"
          style={{ width: `${pct}%` }}
        />
      </div>
      <p className="text-xs text-[var(--color-text-muted)]">
        {formatBytes(volume.usedBytes)} utilisés · {formatBytes(volume.availableBytes)} libres ·{" "}
        {formatBytes(volume.totalBytes)} total
      </p>
      {onSelect ? (
        <button type="button" onClick={onSelect} className="dp-btn-primary mt-auto w-full text-sm">
          {actionLabel}
        </button>
      ) : null}
    </article>
  );
}
