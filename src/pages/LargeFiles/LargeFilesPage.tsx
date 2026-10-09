import { useCallback, useEffect, useState } from "react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { useContextMenu } from "../../components/ContextMenu";
import { EmptyState } from "../../components/EmptyState";
import { InlineAlert } from "../../components/InlineAlert";
import { LoadingBlock } from "../../components/LoadingBlock";
import { FileTypeIcon } from "../../components/FileTypeIcon";
import { PageHeader } from "../../components/PageHeader";
import { copyPathToClipboard, revealFileInExplorer, trashSingleFile } from "../../lib/fileActions";
import { formatUserError } from "../../lib/userError";
import { fetchLargeFiles, fetchSettings } from "../../lib/tauri";
import { formatBytes, formatIsoDate, LARGE_FILE_PRESETS_MB } from "../../lib/format";
import type { LargeFileItem, LargeFilesResult } from "../../types/dto";

export function LargeFilesPage() {
  const { open: openMenu, menuPortal } = useContextMenu();
  const [minBytes, setMinBytes] = useState<number | null>(null);
  const [customMb, setCustomMb] = useState("");
  const [result, setResult] = useState<LargeFilesResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [singleTrashPath, setSingleTrashPath] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(async (threshold: number) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchLargeFiles(threshold);
      setResult(data);
    } catch (err) {
      setError(formatUserError(err));
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSettings()
      .then((response) =>
        setMinBytes(response.settings.largeFileThresholdMb * 1024 * 1024),
      )
      .catch(() => setMinBytes(LARGE_FILE_PRESETS_MB[0].bytes));
  }, []);

  useEffect(() => {
    if (minBytes == null) return;
    void load(minBytes);
  }, [load, minBytes]);

  function applyCustomThreshold() {
    const mb = Number(customMb.replace(",", "."));
    if (!Number.isFinite(mb) || mb <= 0) {
      setError("Seuil personnalisé invalide (Mo > 0).");
      return;
    }
    setError(null);
    setMinBytes(Math.floor(mb * 1024 * 1024));
  }

  function openFileContextMenu(event: React.MouseEvent, file: LargeFileItem) {
    openMenu(
      event,
      [
        {
          id: "reveal",
          label: "Afficher dans l'explorateur",
          onSelect: () => void revealFileInExplorer(file.path).catch(() => {}),
        },
        {
          id: "copy",
          label: "Copier le chemin",
          onSelect: () => void copyPathToClipboard(file.path),
        },
        {
          id: "trash",
          label: "Envoyer à la corbeille",
          danger: true,
          onSelect: () => setSingleTrashPath(file.path),
        },
      ],
      file.name,
    );
  }

  async function confirmSingleTrash() {
    const path = singleTrashPath;
    if (!path) return;
    setSingleTrashPath(null);
    setNotice(null);
    try {
      const trashResult = await trashSingleFile(path);
      if (trashResult.failedCount > 0) {
        setNotice("Impossible d'envoyer ce fichier à la corbeille.");
      } else {
        setNotice("Fichier envoyé à la corbeille.");
        if (minBytes != null) await load(minBytes);
      }
    } catch (err) {
      setError(formatUserError(err));
    }
  }

  return (
    <>
      {menuPortal}
      <PageHeader
        title="Gros fichiers"
        description="Liste triée par taille depuis le dernier scan terminé. Clic droit pour explorateur, copie ou corbeille."
      />

      <div className="flex flex-wrap gap-2">
        {LARGE_FILE_PRESETS_MB.map((preset) => (
          <button
            key={preset.label}
            type="button"
            onClick={() => setMinBytes(preset.bytes)}
            className={[
              "rounded-lg px-3 py-1.5 text-sm transition-colors",
              minBytes === preset.bytes
                ? "bg-[var(--color-accent)] text-white"
                : "border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)]",
            ].join(" ")}
          >
            {preset.label}
          </button>
        ))}
      </div>

      <div className="mt-4 flex flex-wrap items-end gap-2">
        <label className="flex flex-col gap-1 text-xs text-[var(--color-text-muted)]">
          Seuil personnalisé (Mo)
          <input
            type="number"
            min={1}
            value={customMb}
            onChange={(e) => setCustomMb(e.target.value)}
            className="w-36 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-elevated)] px-3 py-2 text-sm text-[var(--color-text)]"
            placeholder="ex. 250"
          />
        </label>
        <button
          type="button"
          onClick={applyCustomThreshold}
          className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text)] hover:border-[var(--color-accent)]"
        >
          Appliquer
        </button>
        <button
          type="button"
          onClick={() => minBytes != null && load(minBytes)}
          disabled={loading || minBytes == null}
          className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
        >
          Actualiser
        </button>
      </div>

      {notice ? (
        <InlineAlert variant="success" className="mt-3">
          {notice}
        </InlineAlert>
      ) : null}

      {error ? (
        <InlineAlert
          variant="error"
          className="mt-3"
          title="Impossible de charger la liste"
          onRetry={() => minBytes != null && void load(minBytes)}
        >
          {error}
        </InlineAlert>
      ) : null}

      {minBytes == null ? (
        <LoadingBlock label="Préparation du seuil…" className="mt-6" />
      ) : !result?.scan && !loading && !error ? (
        <EmptyState
          className="mt-6"
          title="Aucune analyse disponible"
          description="Lancez une analyse de dossier pour indexer les fichiers volumineux."
          actionLabel="Analyser le stockage"
          actionTo="/analyze"
        />
      ) : result?.scan ? (
        <section className="mt-6 space-y-4">
          <p className="text-sm text-[var(--color-text-muted)]">
            Scan #{result.scan.id} — {result.scan.rootPath} — seuil ≥ {formatBytes(result.minSizeBytes ?? 0)} —{" "}
            {(result.matchCount ?? 0).toLocaleString("fr-FR")} fichier(s) —{" "}
            {formatBytes(result.matchTotalBytes ?? 0)} au total
            {(result.matchCount ?? 0) > result.items.length
              ? ` (affichage limité aux ${result.items.length} plus volumineux)`
              : null}
          </p>

          <div className="overflow-hidden rounded-xl border border-[var(--color-border)]">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-[var(--color-surface-elevated)] text-xs uppercase tracking-wide text-[var(--color-text-muted)]">
                <tr>
                  <th className="px-4 py-3 font-medium">Nom</th>
                  <th className="px-4 py-3 font-medium">Taille</th>
                  <th className="px-4 py-3 font-medium">Type</th>
                  <th className="px-4 py-3 font-medium">Emplacement</th>
                  <th className="px-4 py-3 font-medium">Modifié</th>
                </tr>
              </thead>
              <tbody>
                {result.items.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-6 text-[var(--color-text-muted)]">
                      {loading ? "Chargement…" : "Aucun fichier au-dessus de ce seuil."}
                    </td>
                  </tr>
                ) : (
                  result.items.map((file) => (
                    <tr
                      key={file.id}
                      className="border-t border-[var(--color-border)] hover:bg-[var(--color-surface-elevated)]/60"
                      onContextMenu={(e) => openFileContextMenu(e, file)}
                    >
                      <td className="px-4 py-3">
                        <div className="flex max-w-[14rem] items-center gap-2">
                          <FileTypeIcon extension={file.extension} />
                          <span className="truncate text-[var(--color-text)]" title={file.name}>
                            {file.name}
                          </span>
                        </div>
                      </td>
                      <td className="whitespace-nowrap px-4 py-3 text-[var(--color-text)]">
                        {formatBytes(file.sizeBytes ?? 0)}
                      </td>
                      <td className="max-w-[8rem] truncate px-4 py-3 text-[var(--color-text-muted)]" title={file.mimeOrKind ?? file.extension}>
                        {file.mimeOrKind ?? file.extension ?? "—"}
                      </td>
                      <td className="max-w-[18rem] truncate px-4 py-3 font-mono text-xs text-[var(--color-text-muted)]" title={file.path}>
                        {file.path}
                      </td>
                      <td className="whitespace-nowrap px-4 py-3 text-[var(--color-text-muted)]">
                        {formatIsoDate(file.modifiedAt)}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>
      ) : loading ? (
        <LoadingBlock label="Recherche des gros fichiers…" className="mt-6" />
      ) : null}

      <ConfirmDialog
        open={singleTrashPath != null}
        title="Envoyer à la corbeille ?"
        description={singleTrashPath ?? ""}
        confirmLabel="Envoyer à la corbeille"
        onConfirm={() => void confirmSingleTrash()}
        onCancel={() => setSingleTrashPath(null)}
      />
    </>
  );
}
