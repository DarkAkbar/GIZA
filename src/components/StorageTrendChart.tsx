import { formatBytes, formatStorageChartLabel } from "../lib/format";
import type { StorageSnapshotDto } from "../types/dto";

type StorageTrendChartProps = {
  snapshots: StorageSnapshotDto[];
  periodDays: number;
};

const WIDTH = 800;
const HEIGHT = 220;
const PAD = { top: 16, right: 16, bottom: 36, left: 56 };

export function StorageTrendChart({ snapshots, periodDays }: StorageTrendChartProps) {
  if (snapshots.length === 0) return null;

  const innerW = WIDTH - PAD.left - PAD.right;
  const innerH = HEIGHT - PAD.top - PAD.bottom;
  const maxY = Math.max(...snapshots.map((s) => s.totalBytes), 1);

  const xAt = (index: number) => {
    if (snapshots.length <= 1) return PAD.left + innerW / 2;
    return PAD.left + (index / (snapshots.length - 1)) * innerW;
  };
  const yAt = (bytes: number) => PAD.top + innerH - (bytes / maxY) * innerH;

  const usedPoints = snapshots
    .map((s, i) => `${xAt(i)},${yAt(s.usedBytes)}`)
    .join(" ");
  const freePoints = snapshots
    .map((s, i) => `${xAt(i)},${yAt(s.freeBytes)}`)
    .join(" ");

  const yTicks = 4;
  const tickLabels = Array.from({ length: yTicks + 1 }, (_, i) => {
    const value = (maxY * (yTicks - i)) / yTicks;
    return { value, y: yAt(value) };
  });

  const xLabelIndices =
    snapshots.length <= 5
      ? snapshots.map((_, i) => i)
      : [0, Math.floor(snapshots.length / 2), snapshots.length - 1];

  return (
    <div className="w-full overflow-x-auto">
      <svg
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        className="h-auto w-full min-w-[320px] max-w-full"
        role="img"
        aria-label="Graphique d'évolution de l'espace disque"
      >
        {tickLabels.map((tick) => (
          <g key={tick.y}>
            <line
              x1={PAD.left}
              y1={tick.y}
              x2={WIDTH - PAD.right}
              y2={tick.y}
              stroke="var(--color-border)"
              strokeWidth={1}
            />
            <text
              x={PAD.left - 8}
              y={tick.y + 4}
              textAnchor="end"
              className="fill-[var(--color-text-muted)] text-[10px]"
            >
              {formatBytes(Math.round(tick.value))}
            </text>
          </g>
        ))}

        <polyline
          points={freePoints}
          fill="none"
          stroke="var(--color-text-muted)"
          strokeWidth={2}
          strokeLinejoin="round"
          strokeLinecap="round"
          opacity={0.85}
        />
        <polyline
          points={usedPoints}
          fill="none"
          stroke="var(--color-accent)"
          strokeWidth={2.5}
          strokeLinejoin="round"
          strokeLinecap="round"
        />

        {xLabelIndices.map((i) => (
          <text
            key={i}
            x={xAt(i)}
            y={HEIGHT - 8}
            textAnchor="middle"
            className="fill-[var(--color-text-muted)] text-[10px]"
          >
            {formatStorageChartLabel(snapshots[i].createdAt, periodDays)}
          </text>
        ))}
      </svg>
      <div className="mt-2 flex flex-wrap gap-4 text-xs text-[var(--color-text-muted)]">
        <span className="flex items-center gap-2">
          <span className="inline-block h-0.5 w-6 bg-[var(--color-accent)]" aria-hidden />
          Espace utilisé
        </span>
        <span className="flex items-center gap-2">
          <span className="inline-block h-0.5 w-6 bg-[var(--color-text-muted)]" aria-hidden />
          Espace libre
        </span>
      </div>
    </div>
  );
}
