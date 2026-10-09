type ModulePlaceholderProps = {
  module: string;
  mvpPhase: string;
};

export function ModulePlaceholder({ module, mvpPhase }: ModulePlaceholderProps) {
  return (
    <div className="rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-elevated)]/50 p-8 text-center">
      <p className="text-sm font-medium text-[var(--color-text)]">{module}</p>
      <p className="mt-2 text-sm text-[var(--color-text-muted)]">{mvpPhase}</p>
    </div>
  );
}
