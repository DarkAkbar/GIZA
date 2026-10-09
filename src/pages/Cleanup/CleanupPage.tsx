import { useMemo, useState } from "react";
import { InstalledAppsPanel } from "../../components/InstalledAppsPanel";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { useContextMenu } from "../../components/ContextMenu";
import { copyPathToClipboard, revealFileInExplorer, trashSingleFile } from "../../lib/fileActions";
import { EmptyState } from "../../components/EmptyState";
import { InlineAlert } from "../../components/InlineAlert";
import { PageHeader } from "../../components/PageHeader";
import { formatUserError } from "../../lib/userError";
import { analyzeTempFiles, moveFilesToTrash } from "../../lib/tauri";
import { formatBytes, formatIsoDate } from "../../lib/format";
import type { CleanupAnalysisResult, CleanupItem, RiskLevel } from "../../types/dto";

const RISK_FILTERS: { id: "ALL" | RiskLevel; label: string }[] = [
  { id: "ALL", label: "Tous" },
  { id: "SAFE", label: "SAFE" },
  { id: "REVIEW", label: "REVIEW" },
  { id: "RISKY", label: "RISKY" },
];

export function CleanupPage() {
  const { open: openMenu, menuPortal } = useContextMenu();
  const [result, setResult] = useState<CleanupAnalysisResult | null>(null);
  const [singleTrashPath, setSingleTrashPath] = useState<string | null>(null);
  const [riskFilter, setRiskFilter] = useState<"ALL" | RiskLevel>("ALL");
  const [selected, setSelected] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(false);
  const [trashing, setTrashing] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<{ variant: "success" | "warning"; message: string } | null>(
    null,
  );

  const visibleItems = useMemo(() => {
    if (!result) return [];
    if (riskFilter === "ALL") return result.items;
    return result.items.filter((i) => i.riskLevel === riskFilter);
  }, [result, riskFilter]);

  const selectedPaths = useMemo(() => Object.keys(selected), [selected]);
  const selectedBytes = useMemo(
    () => Object.values(selected).reduce((a, b) => a + b, 0),
    [selected],
  );

  async function runAnalysis(clearSelection = true): Promise<CleanupAnalysisResult | null> {
    setLoading(true);
    setError(null);
    setNotice(null);
    if (clearSelection) setSelected({});
    try {
      const data = await analyzeTempFiles();
      setResult(data);
      return data;
    } catch (err) {
      setError(formatUserError(err));
      setResult(null);
      return null;
    } finally {
      setLoading(false);
    }
  }

  function applySelectionFromResult(
    data: CleanupAnalysisResult,
    levels: RiskLevel[],
  ) {
    const next: Record<string, number> = {};
    for (const item of data.items) {
      if (levels.includes(item.riskLevel)) next[item.path] = item.sizeBytes;
    }
    setSelected(next);
  }

  async function analyzeAndSelectSafe() {
    const data = await runAnalysis(false);
    if (data) applySelectionFromResult(data, ["SAFE"]);
  }

  function toggleItem(item: CleanupItem, checked: boolean) {
    setSelected((prev) => {
      const next = { ...prev };
      if (checked) next[item.path] = item.sizeBytes;
      else delete next[item.path];
      return next;
    });
  }

  function selectSafeOnly() {
    if (!result) return;
    applySelectionFromResult(result, ["SAFE"]);
  }

  function selectSafeAndReview() {
    if (!result) return;
    applySelectionFromResult(result, ["SAFE", "REVIEW"]);
  }

  async function confirmTrash() {
    setConfirmOpen(false);
    setTrashing(true);
    setError(null);
    setNotice(null);
    try {
      const trashResult = await moveFilesToTrash(selectedPaths);
      if (trashResult.failedCount > 0) {
        setNotice({
          variant: "warning",
          message: `${trashResult.successCount} envoyé(s), ${trashResult.failedCount} échec(s). Consultez l'historique pour le détail.`,
        });
      } else if (trashResult.successCount > 0) {
        setNotice({
          variant: "success",
          message: `${trashResult.successCount} fichier(s) envoyé(s) à la corbeille.`,
        });
      }
      setSelected({});
      await runAnalysis();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setTrashing(false);
    }
  }

  const hasRiskySelected = useMemo(
    () =>
      result?.items.some((i) => i.path in selected && i.riskLevel === "RISKY") ?? false,
    [result, selected],
  );

  function openCleanupContextMenu(event: React.MouseEvent, item: CleanupItem) {
    openMenu(
      event,
      [
        {
          id: "toggle",
          label: item.path in selected ? "Décocher" : "Cocher",
          onSelect: () => toggleItem(item, !(item.path in selected)),
        },
        {
          id: "reveal",
          label: "Afficher dans l'explorateur",
          onSelect: () => void revealFileInExplorer(item.path).catch(() => {}),
        },
        {
          id: "copy",
          label: "Copier le chemin",
          onSelect: () => void copyPathToClipboard(item.path),
        },
        {
          id: "trash",
          label: "Envoyer à la corbeille",
          danger: true,
          onSelect: () => setSingleTrashPath(item.path),
        },
      ],
      item.categoryLabel,
    );
  }

  async function confirmSingleTrash() {
    if (!singleTrashPath) return;
    const path = singleTrashPath;
    setSingleTrashPath(null);
    setTrashing(true);
    try {
      await trashSingleFile(path);
      setNotice({ variant: "success", message: "Fichier envoyé à la corbeille." });
      await runAnalysis();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setTrashing(false);
    }
  }

  return (
    <>
      {menuPortal}
      <PageHeader
        title="Nettoyage"
        description="Fichiers temporaires et caches courants. Aucune suppression automatique — corbeille uniquement après confirmation."
      />

      <div className="flex flex-wrap gap-3">
        <button
          type="button"
          onClick={() => void runAnalysis()}
          disabled={loading}
          className="dp-btn-primary"
        >
          {loading ? "Analyse…" : "Analyser les temporaires"}
        </button>
        <button
          type="button"
          onClick={() => void analyzeAndSelectSafe()}
          disabled={loading}
          className="dp-btn-secondary"
        >
          Analyser + tout sélectionner (SAFE)
        </button>
        <button
          type="button"
          onClick={selectSafeOnly}
          disabled={!result?.items.length}
          className="dp-btn-secondary"
        >
          Tout sélectionner (SAFE)
        </button>
        <button
          type="button"
          onClick={selectSafeAndReview}
          disabled={!result?.items.length}
          className="dp-btn-secondary"
        >
          Tout sélectionner (SAFE + REVIEW)
        </button>
        <button
          type="button"
          disabled={selectedPaths.length === 0 || trashing}
          onClick={() => setConfirmOpen(true)}
          className="rounded-lg border border-amber-500/50 px-4 py-2 text-sm text-amber-200 hover:bg-amber-500/10 disabled:opacity-50"
        >
          Corbeille ({selectedPaths.length})
        </button>
      </div>

      <InlineAlert variant="info" className="mt-3">
        Les fichiers RISKY ne sont jamais présélectionnés. « SAFE + REVIEW » inclut des caches à
        vérifier — la corbeille reste confirmée manuellement.
      </InlineAlert>

      <div className="mt-4 flex flex-wrap gap-2">
        {RISK_FILTERS.map((f) => (
          <button
            key={f.id}
            type="button"
            onClick={() => setRiskFilter(f.id)}
            className={[
              "rounded-lg px-3 py-1.5 text-sm",
              riskFilter === f.id
                ? "bg-[var(--color-accent)] text-white"
                : "border border-[var(--color-border)] text-[var(--color-text-muted)]",
            ].join(" ")}
          >
            {f.label}
          </button>
        ))}
      </div>

      {error ? (
        <InlineAlert variant="error" className="mt-3" title="Erreur">
          {error}
        </InlineAlert>
      ) : null}
      {notice ? (
        <InlineAlert variant={notice.variant} className="mt-3">
          {notice.message}
        </InlineAlert>
      ) : null}

      {result ? (
        <section className="mt-6 space-y-4 text-sm">
          <p className="text-[var(--color-text-muted)]">
            {result.items.length.toLocaleString("fr-FR")} fichier(s) — dossiers :{" "}
            {result.rootsScanned.map((r) => r.replace(/\\$/, "")).join(", ")}
            {result.truncated ? " (liste tronquée)" : null}
          </p>
          {result.categories.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {result.categories.map((c) => (
                <span
                  key={c.category}
                  className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs text-[var(--color-text-muted)]"
                >
                  {c.label} — {formatBytes(c.totalBytes)} —{" "}
                  <RiskBadge level={c.riskLevel} />
                </span>
              ))}
            </div>
          ) : null}
          <div className="overflow-hidden rounded-xl border border-[var(--color-border)]">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-[var(--color-surface-elevated)] text-xs uppercase text-[var(--color-text-muted)]">
                <tr>
                  <th className="w-10 px-3 py-3" />
                  <th className="px-3 py-3">Fichier</th>
                  <th className="px-3 py-3">Catégorie</th>
                  <th className="px-3 py-3">Risque</th>
                  <th className="px-3 py-3">Taille</th>
                  <th className="px-3 py-3">Modifié</th>
                </tr>
              </thead>
              <tbody>
                {visibleItems.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="px-4 py-6 text-[var(--color-text-muted)]">
                      Aucun fichier pour ce filtre.
                    </td>
                  </tr>
                ) : (
                  visibleItems.map((item) => (
                    <tr
                      key={item.path}
                      className="border-t border-[var(--color-border)]"
                      onContextMenu={(e) => openCleanupContextMenu(e, item)}
                    >
                      <td className="px-3 py-2">
                        <input
                          type="checkbox"
                          checked={item.path in selected}
                          onChange={(e) => toggleItem(item, e.target.checked)}
                        />
                      </td>
                      <td
                        className="max-w-[16rem] truncate px-3 py-2 font-mono text-xs"
                        title={item.path}
                      >
                        {item.path}
                      </td>
                      <td className="px-3 py-2 text-[var(--color-text-muted)]">{item.categoryLabel}</td>
                      <td className="px-3 py-2">
                        <RiskBadge level={item.riskLevel} />
                      </td>
                      <td className="px-3 py-2">{formatBytes(item.sizeBytes)}</td>
                      <td className="px-3 py-2 text-[var(--color-text-muted)]">
                        {formatIsoDate(item.modifiedAt)}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>
      ) : (
        <EmptyState
          className="mt-6"
          title="Inventaire non lancé"
          description="Analysez les temporaires utilisateur et caches navigateur. Les zones sensibles sont classées RISKY — rien n'est supprimé sans votre confirmation."
          actionLabel="Analyser les temporaires"
          onAction={() => void runAnalysis()}
        />
      )}

      {selectedPaths.length > 0 ? (
        <p className="mt-3 text-sm text-[var(--color-text-muted)]">
          Sélection : {selectedPaths.length} — {formatBytes(selectedBytes)}
        </p>
      ) : null}

      <InstalledAppsPanel className="mt-8" limit={18} />

      <ConfirmDialog
        open={singleTrashPath != null}
        title="Envoyer à la corbeille ?"
        description={singleTrashPath ?? ""}
        confirmLabel="Envoyer à la corbeille"
        onConfirm={() => void confirmSingleTrash()}
        onCancel={() => setSingleTrashPath(null)}
      />

      <ConfirmDialog
        open={confirmOpen}
        title="Envoyer à la corbeille ?"
        description={`${selectedPaths.length} fichier(s) temporaire(s) (${formatBytes(selectedBytes)}).`}
        warning={
          hasRiskySelected
            ? "Des fichiers RISKY sont inclus (zones sensibles). Vérifiez la sélection."
            : undefined
        }
        confirmLabel="Envoyer à la corbeille"
        onConfirm={() => void confirmTrash()}
        onCancel={() => setConfirmOpen(false)}
      />
    </>
  );
}

function RiskBadge({ level }: { level: RiskLevel }) {
  const styles: Record<RiskLevel, string> = {
    SAFE: "text-emerald-400 border-emerald-500/40",
    REVIEW: "text-amber-300 border-amber-500/40",
    RISKY: "text-red-400 border-red-500/40",
  };
  return (
    <span className={`rounded border px-1.5 py-0.5 text-[10px] font-medium uppercase ${styles[level]}`}>
      {level}
    </span>
  );
}
