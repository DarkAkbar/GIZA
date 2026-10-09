type ProgressStatusProps = {
  label: string;
  detail?: string;
  active?: boolean;
};

/** Indication visuelle d'une opération en cours ou d'un état prêt. */
export function ProgressStatus({ label, detail, active }: ProgressStatusProps) {
  return (
    <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-wide text-[var(--color-text-muted)]">
      {active ? (
        <span
          className="inline-block h-2 w-2 animate-pulse rounded-full bg-[var(--color-accent)]"
          aria-hidden
        />
      ) : null}
      <span>{label}</span>
      {detail ? <span className="normal-case text-[var(--color-text-muted)]">— {detail}</span> : null}
    </div>
  );
}
