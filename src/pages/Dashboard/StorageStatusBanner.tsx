import type { StorageStatusSummary } from "./dashboardSummary";
import { statusSurfaceClass } from "./dashboardSummary";

type StorageStatusBannerProps = {
  summary: StorageStatusSummary;
  loading?: boolean;
};

export function StorageStatusBanner({ summary, loading }: StorageStatusBannerProps) {
  if (loading) {
    return (
      <div
        className="animate-pulse rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-4"
        aria-hidden
      >
        <div className="h-4 w-48 rounded bg-[var(--color-border)]" />
        <div className="mt-2 h-3 w-full max-w-md rounded bg-[var(--color-border)]/70" />
      </div>
    );
  }

  return (
    <div
      className={`rounded-xl border p-4 ${statusSurfaceClass(summary.level)}`}
      role="status"
      aria-live="polite"
    >
      <p className="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">
        État · {summary.label}
      </p>
      <p className="mt-1 text-base font-medium text-[var(--color-text)]">{summary.message}</p>
    </div>
  );
}
