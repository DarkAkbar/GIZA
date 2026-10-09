import { useCallback, useEffect, useMemo, useState } from "react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { EmptyState } from "../../components/EmptyState";
import { InlineAlert } from "../../components/InlineAlert";
import { LoadingBlock } from "../../components/LoadingBlock";
import { PageHeader } from "../../components/PageHeader";
import { formatUserError } from "../../lib/userError";
import { applyOrganizeMoves, fetchOrganizeProposals } from "../../lib/tauri";
import { formatBytes } from "../../lib/format";
import type { OrganizeProposalsResult, ProposedMove } from "../../types/dto";

export function OrganizePage() {
  const [proposals, setProposals] = useState<OrganizeProposalsResult | null>(null);
  const [selected, setSelected] = useState<Record<string, ProposedMove>>({});
  const [loading, setLoading] = useState(false);
  const [applying, setApplying] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<{ variant: "success" | "warning"; message: string } | null>(
    null,
  );

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    setNotice(null);
    try {
      const data = await fetchOrganizeProposals();
      setProposals(data);
      setSelected({});
    } catch (err) {
      setError(formatUserError(err));
      setProposals(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const selectedMoves = useMemo(() => Object.values(selected), [selected]);
  const selectedBytes = useMemo(
    () => selectedMoves.reduce((s, m) => s + m.sizeBytes, 0),
    [selectedMoves],
  );

  const hasConflict = useMemo(
    () => selectedMoves.some((m) => m.conflict),
    [selectedMoves],
  );

  function toggleMove(move: ProposedMove, checked: boolean) {
    setSelected((prev) => {
      const next = { ...prev };
      if (checked) next[move.sourcePath] = move;
      else delete next[move.sourcePath];
      return next;
    });
  }

  function selectAllClear() {
    if (!proposals?.moves.length) return;
    const next: Record<string, ProposedMove> = {};
    for (const move of proposals.moves) {
      if (!move.conflict) next[move.sourcePath] = move;
    }
    setSelected(next);
  }

  function clearSelection() {
    setSelected({});
  }

  async function confirmApply() {
    setConfirmOpen(false);
    setApplying(true);
    setError(null);
    setNotice(null);
    try {
      const result = await applyOrganizeMoves(
        selectedMoves.map((m) => ({ sourcePath: m.sourcePath, destPath: m.destPath })),
      );
      if (result.failedCount > 0) {
        setNotice({
          variant: "warning",
          message: `${result.successCount} déplacé(s), ${result.failedCount} échec(s).`,
        });
      } else if (result.successCount > 0) {
        setNotice({
          variant: "success",
          message: `${result.successCount} fichier(s) déplacé(s) avec succès.`,
        });
      }
      await load();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setApplying(false);
    }
  }

  return (
    <>
      <PageHeader
        title="Organisation"
        description="Classe les fichiers à la racine du dossier scanné (ex. Téléchargements) par type. Prévisualisation obligatoire avant déplacement."
      />

      <div className="flex flex-wrap gap-3">
        <button
          type="button"
          onClick={() => void load()}
          disabled={loading}
          className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
        >
          Actualiser
        </button>
        <button
          type="button"
          onClick={selectAllClear}
          disabled={!proposals?.moves.length}
          className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
        >
          Tout accepter (sans conflit)
        </button>
        <button
          type="button"
          onClick={clearSelection}
          disabled={selectedMoves.length === 0}
          className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
        >
          Tout refuser
        </button>
        <button
          type="button"
          disabled={selectedMoves.length === 0 || applying || hasConflict}
          onClick={() => setConfirmOpen(true)}
          className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
        >
          Appliquer ({selectedMoves.length})
        </button>
      </div>

      {error ? (
        <InlineAlert variant="error" className="mt-3" title="Erreur" onRetry={() => void load()}>
          {error}
        </InlineAlert>
      ) : null}
      {notice ? (
        <InlineAlert variant={notice.variant} className="mt-3">
          {notice.message}
        </InlineAlert>
      ) : null}

      {loading && !proposals ? (
        <LoadingBlock label="Chargement des propositions…" className="mt-6" />
      ) : !proposals?.scan ? (
        <EmptyState
          className="mt-6"
          title="Aucun scan terminé"
          description="Analysez un dossier (par ex. Téléchargements) pour obtenir des propositions de classement."
          actionLabel="Analyser un dossier"
          actionTo="/analyze"
        />
      ) : (
        <section className="mt-6 space-y-4 text-sm">
          <p className="text-[var(--color-text-muted)]">
            Scan #{proposals.scan.id} — {proposals.rootPath} —{" "}
            {proposals.moveCount.toLocaleString("fr-FR")} proposition(s) —{" "}
            {formatBytes(proposals.totalBytes)}
          </p>
          <p className="text-xs text-[var(--color-text-muted)]">
            Seuls les fichiers directement à la racine du dossier scanné sont proposés. Les sous-dossiers
            existants ne sont pas modifiés automatiquement.
          </p>

          <div className="overflow-hidden rounded-xl border border-[var(--color-border)]">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-[var(--color-surface-elevated)] text-xs uppercase text-[var(--color-text-muted)]">
                <tr>
                  <th className="w-10 px-3 py-3" />
                  <th className="px-3 py-3">Source</th>
                  <th className="px-3 py-3">Destination</th>
                  <th className="px-3 py-3">Type</th>
                  <th className="px-3 py-3">Taille</th>
                </tr>
              </thead>
              <tbody>
                {proposals.moves.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-6 text-[var(--color-text-muted)]">
                      {loading
                        ? "Chargement…"
                        : "Aucun fichier en vrac à la racine — déjà organisé ou dossier vide."}
                    </td>
                  </tr>
                ) : (
                  proposals.moves.map((move) => (
                    <tr
                      key={move.sourcePath}
                      className={[
                        "border-t border-[var(--color-border)]",
                        move.conflict ? "bg-red-500/5" : "",
                      ].join(" ")}
                    >
                      <td className="px-3 py-2">
                        <input
                          type="checkbox"
                          disabled={move.conflict}
                          checked={move.sourcePath in selected}
                          onChange={(e) => toggleMove(move, e.target.checked)}
                        />
                      </td>
                      <td className="max-w-[12rem] truncate px-3 py-2 font-mono text-xs" title={move.sourcePath}>
                        {move.fileName}
                      </td>
                      <td className="max-w-[18rem] truncate px-3 py-2 font-mono text-xs text-emerald-400/90" title={move.destPath}>
                        {move.destPath}
                        {move.conflict ? (
                          <span className="ml-2 text-red-400">(existe déjà)</span>
                        ) : null}
                      </td>
                      <td className="px-3 py-2 text-[var(--color-text-muted)]">{move.categoryLabel}</td>
                      <td className="px-3 py-2">{formatBytes(move.sizeBytes)}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>
      )}

      {selectedMoves.length > 0 ? (
        <p className="mt-3 text-sm text-[var(--color-text-muted)]">
          {selectedMoves.length} déplacement(s) — {formatBytes(selectedBytes)}
        </p>
      ) : null}

      <ConfirmDialog
        open={confirmOpen}
        title="Appliquer les déplacements ?"
        description={`${selectedMoves.length} fichier(s) seront déplacés dans des sous-dossiers par type. Cette action est journalisée (type « organisation »).`}
        confirmDisabled={hasConflict}
        warning={
          hasConflict
            ? "Désélectionnez les lignes en conflit (destination déjà existante)."
            : undefined
        }
        confirmLabel="Déplacer les fichiers"
        onConfirm={() => void confirmApply()}
        onCancel={() => setConfirmOpen(false)}
      />
    </>
  );
}
