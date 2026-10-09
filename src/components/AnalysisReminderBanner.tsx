import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { InlineAlert } from "./InlineAlert";
import { ANALYSIS_REFRESH_EVENT, fetchAnalysisScheduleStatus, requestVolumeRescan } from "../lib/tauri";
import { formatUserError } from "../lib/userError";
import type { AnalysisScheduleStatus } from "../types/dto";

export function AnalysisReminderBanner() {
  const [status, setStatus] = useState<AnalysisScheduleStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await fetchAnalysisScheduleStatus());
    } catch {
      setStatus(null);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const onAnalysis = () => void refresh();
    window.addEventListener(ANALYSIS_REFRESH_EVENT, onAnalysis);
    return () => window.removeEventListener(ANALYSIS_REFRESH_EVENT, onAnalysis);
  }, [refresh]);

  async function runRescan(forceAll: boolean) {
    setLoading(true);
    setError(null);
    setNotice(null);
    try {
      const result = await requestVolumeRescan(forceAll);
      setNotice(result.message);
      await refresh();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setLoading(false);
    }
  }

  if (!status?.reminderEnabled) return null;

  if (!status.hasEverScanned) {
    return (
      <InlineAlert variant="info" className="mb-4">
        Aucune analyse terminée pour l&apos;instant.{" "}
        <Link to="/analyze" className="font-medium text-orange-400 hover:underline">
          Analyser un dossier
        </Link>{" "}
        ou attendez la fin de l&apos;analyse automatique au démarrage.
      </InlineAlert>
    );
  }

  if (!status.stale) return null;

  const days = status.daysSinceLastScan ?? status.intervalDays;

  return (
    <div className="mb-4 rounded-xl border border-sky-500/30 bg-sky-500/10 px-4 py-3">
      <p className="text-sm font-medium text-[var(--color-text)]">Ré-analyse conseillée</p>
      <p className="mt-1 text-sm text-[var(--color-text-muted)]">
        Dernière indexation il y a environ {days} jour(s) (seuil : {status.intervalDays} j). Giza
        ne nettoie rien automatiquement — vous choisissez les actions après la mise à jour.
      </p>
      {error ? (
        <p className="mt-2 text-xs text-red-400">{error}</p>
      ) : null}
      {notice ? <p className="mt-2 text-xs text-emerald-400/90">{notice}</p> : null}
      <div className="mt-3 flex flex-wrap gap-2">
        <button
          type="button"
          disabled={loading}
          onClick={() => void runRescan(false)}
          className="rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
        >
          {loading ? "Planification…" : "Relancer (volumes obsolètes)"}
        </button>
        <button
          type="button"
          disabled={loading}
          onClick={() => void runRescan(true)}
          className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
        >
          Tout relancer
        </button>
        <Link
          to="/settings"
          className="rounded-lg px-3 py-1.5 text-sm text-orange-400/90 hover:underline"
        >
          Fréquence du rappel
        </Link>
      </div>
    </div>
  );
}
