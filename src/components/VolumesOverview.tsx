import { useCallback, useEffect, useState } from "react";
import { LoadingBlock } from "./LoadingBlock";
import { InlineAlert } from "./InlineAlert";
import { VolumeCard } from "./VolumeCard";
import { fetchMountedVolumes, fetchScanRootSuggestions } from "../lib/tauri";
import { formatUserError } from "../lib/userError";
import type { ScanRootSuggestion, VolumeStats } from "../types/dto";

type VolumesOverviewProps = {
  title?: string;
  description?: string;
  className?: string;
  onPickScanRoot?: (suggestion: ScanRootSuggestion) => void;
  selectedMountPath?: string;
  showAnalyzeButtons?: boolean;
  /** Retarde le chargement pour laisser respirer l’UI au montage de la page. */
  deferLoadMs?: number;
};

export function VolumesOverview({
  title = "Tous vos disques",
  description = "Vue d'ensemble sans scan de fichiers — choisissez un volume pour l'analyse.",
  className = "",
  onPickScanRoot,
  selectedMountPath,
  showAnalyzeButtons = true,
  deferLoadMs = 0,
}: VolumesOverviewProps) {
  const [volumes, setVolumes] = useState<VolumeStats[]>([]);
  const [suggestions, setSuggestions] = useState<ScanRootSuggestion[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [vols, roots] = await Promise.all([
        fetchMountedVolumes(),
        fetchScanRootSuggestions(),
      ]);
      setVolumes(vols);
      setSuggestions(roots);
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(), deferLoadMs);
    return () => window.clearTimeout(timer);
  }, [deferLoadMs, load]);

  const volumeByMount = (mount: string) =>
    volumes.find((v) => v.mountPath.toLowerCase() === mount.toLowerCase());

  return (
    <section className={`dp-card p-5 ${className}`.trim()}>
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h2 className="text-sm font-medium text-[var(--color-text)]">{title}</h2>
          <p className="mt-1 text-xs text-[var(--color-text-muted)]">{description}</p>
        </div>
        <button type="button" onClick={() => void load()} className="dp-btn-ghost text-xs">
          Actualiser
        </button>
      </div>

      {loading ? (
        <LoadingBlock label="Lecture des volumes…" className="mt-4" />
      ) : error ? (
        <InlineAlert variant="error" className="mt-4" onRetry={() => void load()}>
          {error}
        </InlineAlert>
      ) : (
        <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {suggestions.map((s) => {
            const vol = volumeByMount(s.mountPath);
            if (!vol) return null;
            return (
              <VolumeCard
                key={s.mountPath}
                volume={vol}
                selected={selectedMountPath?.toLowerCase() === s.mountPath.toLowerCase()}
                compact
                actionLabel={
                  showAnalyzeButtons ? `Choisir — ${s.label.split(" (")[0]}` : undefined
                }
                onSelect={
                  showAnalyzeButtons && onPickScanRoot
                    ? () => onPickScanRoot(s)
                    : undefined
                }
              />
            );
          })}
        </div>
      )}
    </section>
  );
}
