import { Link } from "react-router-dom";
import { InlineAlert } from "../../components/InlineAlert";
import { RecommendationTypeIcon } from "../../components/icons/NavIcons";
import { formatBytes } from "../../lib/format";
import type { RecommendationDto } from "../../types/dto";

function severityStyles(severity: string): string {
  switch (severity) {
    case "important":
      return "border-amber-500/40 bg-amber-500/10";
    case "warning":
      return "border-yellow-500/30 bg-yellow-500/5";
    default:
      return "border-[var(--color-border)] bg-[var(--color-surface)]";
  }
}

function RecommendationCard({ item }: { item: RecommendationDto }) {
  return (
    <Link
      to={item.actionRoute}
      className={`block rounded-xl border p-4 transition-colors hover:border-[var(--color-accent)]/45 ${severityStyles(item.severity)}`}
    >
      <div className="flex gap-3">
        <span className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-black/20">
          <RecommendationTypeIcon type={item.recommendationType} />
        </span>
        <div className="min-w-0 flex-1">
          <h3 className="text-sm font-medium text-[var(--color-text)]">{item.title}</h3>
          <p className="mt-1 text-sm text-[var(--color-text)]">{item.description}</p>
          <p className="mt-2 text-xs text-[var(--color-text-muted)]">{item.reason}</p>
          {item.potentialBytes > 0 ? (
            <p className="mt-1 text-xs text-[var(--color-text-muted)]">
              Volume concerné : {formatBytes(item.potentialBytes)}
            </p>
          ) : null}
          <span className="mt-3 inline-block text-sm font-medium text-[var(--color-accent)]">
            {item.actionLabel} →
          </span>
        </div>
      </div>
    </Link>
  );
}

export type RecommendationsSectionProps = {
  items: RecommendationDto[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
};

export function RecommendationsSection({
  items,
  loading,
  error,
  onRetry,
}: RecommendationsSectionProps) {
  const hasImportant = items.some((i) => i.severity === "important");

  return (
    <section
      className={[
        "rounded-xl border bg-[var(--color-surface-elevated)] p-5",
        hasImportant ? "border-amber-500/40" : "border-[var(--color-border)]",
      ].join(" ")}
      aria-labelledby="dashboard-recommendations-heading"
    >
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h2
            id="dashboard-recommendations-heading"
            className="text-sm font-medium text-[var(--color-text)]"
          >
            Solutions proposées
          </h2>
          <p className="mt-1 text-xs text-[var(--color-text-muted)]">
            Doublons, gros fichiers, temporaires et apps — rien n&apos;est supprimé sans vous.
          </p>
        </div>
      </div>

      {loading ? (
        <p className="mt-4 text-sm text-[var(--color-text-muted)]">Analyse des signaux…</p>
      ) : error ? (
        <InlineAlert variant="warning" className="mt-4" title="Recommandations indisponibles" onRetry={onRetry}>
          {error}
        </InlineAlert>
      ) : items.length === 0 ? (
        <InlineAlert variant="info" className="mt-4">
          Aucune recommandation pour le moment.
        </InlineAlert>
      ) : (
        <ul className="mt-4 flex flex-col gap-3">
          {items.map((item) => (
            <li key={item.id}>
              <RecommendationCard item={item} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
