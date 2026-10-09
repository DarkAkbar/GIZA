import { Link } from "react-router-dom";

type EmptyStateProps = {
  title: string;
  description: string;
  actionLabel?: string;
  actionTo?: string;
  onAction?: () => void;
  className?: string;
};

export function EmptyState({
  title,
  description,
  actionLabel,
  actionTo,
  onAction,
  className = "",
}: EmptyStateProps) {
  return (
    <div
      className={`rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-elevated)]/50 px-6 py-8 text-center ${className}`}
    >
      <p className="text-sm font-medium text-[var(--color-text)]">{title}</p>
      <p className="mx-auto mt-2 max-w-md text-sm text-[var(--color-text-muted)]">{description}</p>
      {actionLabel && actionTo ? (
        <Link
          to={actionTo}
          className="mt-4 inline-block rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
        >
          {actionLabel}
        </Link>
      ) : null}
      {actionLabel && onAction && !actionTo ? (
        <button
          type="button"
          onClick={onAction}
          className="mt-4 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
        >
          {actionLabel}
        </button>
      ) : null}
    </div>
  );
}
