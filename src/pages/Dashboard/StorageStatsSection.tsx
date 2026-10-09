import { Link } from "react-router-dom";
import { formatBytes, formatIsoDate } from "../../lib/format";
import type { StorageInsights } from "../../types/dto";

type StorageStatsSectionProps = {
  insights: StorageInsights | null;
  insightsLoading: boolean;
  insightsError: string | null;
  largeFileThresholdMb: number;
};

export function StorageStatsSection({
  insights,
  insightsLoading,
  insightsError,
  largeFileThresholdMb,
}: StorageStatsSectionProps) {
  return (
    <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
      <h2 className="text-sm font-medium text-[var(--color-text)]">Statistiques</h2>
      <p className="mt-1 text-xs text-[var(--color-text-muted)]">
        Basées sur le dernier scan indexé — cliquez une carte pour ouvrir la page correspondante.
      </p>

      {insightsLoading ? (
        <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-3 animate-pulse">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-20 rounded-lg bg-[var(--color-border)]/60" />
          ))}
        </div>
      ) : insightsError ? (
        <p className="mt-4 text-sm text-amber-400/90">{insightsError}</p>
      ) : !insights?.scan ? (
        <div className="mt-4 text-sm text-[var(--color-text-muted)]">
          <p className="font-medium text-[var(--color-text)]">
            Giza n&apos;a pas encore suffisamment de données d&apos;analyse.
          </p>
          <p className="mt-2">
            Effectuez une analyse pour obtenir des statistiques détaillées (fichiers, doublons, gros
            fichiers).
          </p>
          <Link
            to="/analyze"
            className="mt-3 inline-block rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
          >
            Analyser le stockage
          </Link>
        </div>
      ) : (
        <dl className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <Stat
            label="Fichiers analysés"
            value={insights.scan.filesSeen.toLocaleString("fr-FR")}
            hint={formatBytes(insights.scan.bytesTotal) + " indexés"}
            to="/analyze"
          />
          <Stat
            label={`Gros fichiers (≥ ${largeFileThresholdMb.toLocaleString("fr-FR")} Mo)`}
            value={insights.largeFilesCount.toLocaleString("fr-FR")}
            hint={
              insights.largeFilesCount > 0
                ? `${formatBytes(insights.largeFilesTotalBytes)} — ouvrir la liste`
                : "Aucun au-dessus du seuil"
            }
            to="/large-files"
          />
          <Stat
            label="Groupes de doublons"
            value={
              insights.hasDuplicateAnalysis
                ? insights.duplicateGroupCount.toLocaleString("fr-FR")
                : "—"
            }
            hint={
              !insights.hasDuplicateAnalysis
                ? "Analyse doublons non effectuée"
                : insights.duplicateGroupCount === 0
                  ? "Aucun groupe détecté"
                  : "Examiner et fusionner"
            }
            to="/duplicates"
          />
          {insights.hasDuplicateAnalysis && insights.duplicateRecoverableBytes > 0 ? (
            <Stat
              label="Espace récupérable (doublons)"
              value={formatBytes(insights.duplicateRecoverableBytes)}
              hint="En conservant une copie par groupe"
              accent
              to="/duplicates"
            />
          ) : null}
          <Stat
            label="Dernière analyse"
            value={formatLastScan(insights.scan.finishedAt)}
            hint={`Scan #${insights.scan.id} — ${insights.scan.rootPath}`}
            to="/history"
          />
        </dl>
      )}
    </section>
  );
}

function formatLastScan(finishedAt?: string): string {
  if (!finishedAt) return "Terminée (date inconnue)";
  const date = new Date(finishedAt);
  if (Number.isNaN(date.getTime())) return formatIsoDate(finishedAt);
  const now = new Date();
  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  const time = date.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  if (sameDay) return `Aujourd'hui à ${time}`;
  return formatIsoDate(finishedAt);
}

function Stat({
  label,
  value,
  hint,
  accent,
  to,
}: {
  label: string;
  value: string;
  hint: string;
  accent?: boolean;
  to?: string;
}) {
  const inner = (
    <>
      <dt className="text-xs text-[var(--color-text-muted)]">{label}</dt>
      <dd
        className={[
          "mt-1 text-lg font-semibold",
          accent ? "text-emerald-400" : "text-[var(--color-text)]",
        ].join(" ")}
      >
        {value}
      </dd>
      <dd className="mt-1 text-xs text-[var(--color-text-muted)]">{hint}</dd>
    </>
  );

  if (!to) {
    return (
      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3">
        {inner}
      </div>
    );
  }

  return (
    <Link
      to={to}
      className="block rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 transition-colors hover:border-orange-500/40 hover:bg-[var(--color-surface-elevated)]"
    >
      {inner}
    </Link>
  );
}
