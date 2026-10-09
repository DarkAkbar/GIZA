type LoadingBlockProps = {
  label?: string;
  className?: string;
};

export function LoadingBlock({ label = "Chargement…", className = "" }: LoadingBlockProps) {
  return (
    <div className={`animate-pulse space-y-3 ${className}`} aria-busy="true" aria-live="polite">
      <p className="text-sm text-[var(--color-text-muted)]">{label}</p>
      <div className="h-2 max-w-xs rounded bg-[var(--color-border)]" />
      <div className="h-24 rounded-xl bg-[var(--color-border)]/60" />
    </div>
  );
}
