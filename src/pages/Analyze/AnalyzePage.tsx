import { useCallback, useEffect, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { InlineAlert } from "../../components/InlineAlert";
import { PageHeader } from "../../components/PageHeader";
import { ProgressStatus } from "../../components/ProgressStatus";
import { formatUserError } from "../../lib/userError";
import {
  cancelScan,
  getScanStatus,
  onScanFinished,
  onScanProgress,
  startScan,
} from "../../lib/tauri";
import { formatBytes } from "../../lib/format";
import { VolumesOverview } from "../../components/VolumesOverview";
import type { ScanRootSuggestion, ScanSnapshot } from "../../types/dto";

const ACTIVE_STATUSES = new Set(["running", "cancelling"]);

export function AnalyzePage() {
  const location = useLocation();
  const consumedNavState = useRef(false);
  const [folder, setFolder] = useState<string | null>(null);
  const [selectedMount, setSelectedMount] = useState<string | undefined>();
  const [selectedRoot, setSelectedRoot] = useState<ScanRootSuggestion | null>(null);
  const [scan, setScan] = useState<ScanSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [volumeConfirmOpen, setVolumeConfirmOpen] = useState(false);
  const progressFlushTimer = useRef<number | null>(null);
  const latestProgress = useRef<{
    scanId: number;
    filesSeen: number;
    bytesTotal: number;
    currentPath?: string;
    status: string;
  } | null>(null);

  const applySnapshot = useCallback((snapshot: ScanSnapshot) => {
    setScan(snapshot);
  }, []);

  useEffect(() => {
    getScanStatus()
      .then(applySnapshot)
      .catch(() => setScan(null));

    let unlistenProgress: (() => void) | undefined;
    let unlistenFinished: (() => void) | undefined;

    void (async () => {
      unlistenProgress = await onScanProgress((payload) => {
        latestProgress.current = payload;
        if (progressFlushTimer.current !== null) return;
        progressFlushTimer.current = window.setTimeout(() => {
          progressFlushTimer.current = null;
          const p = latestProgress.current;
          if (!p) return;
          setScan((prev) => ({
            scanId: p.scanId,
            rootPath: prev?.rootPath,
            status: p.status,
            filesSeen: p.filesSeen,
            bytesTotal: p.bytesTotal,
            currentPath: p.currentPath,
            errorMessage: prev?.errorMessage,
          }));
        }, 250);
      });
      unlistenFinished = await onScanFinished((payload) => {
        if (progressFlushTimer.current !== null) {
          window.clearTimeout(progressFlushTimer.current);
          progressFlushTimer.current = null;
        }
        latestProgress.current = null;
        setScan((prev) => ({
          scanId: payload.scanId,
          rootPath: prev?.rootPath,
          status: payload.status,
          filesSeen: payload.filesSeen,
          bytesTotal: payload.bytesTotal,
          currentPath: undefined,
          errorMessage: payload.errorMessage,
        }));
      });
    })();

    return () => {
      if (progressFlushTimer.current !== null) {
        window.clearTimeout(progressFlushTimer.current);
      }
      unlistenProgress?.();
      unlistenFinished?.();
    };
  }, [applySnapshot]);

  async function pickFolder() {
    setError(null);
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choisir un dossier à analyser",
    });
    if (selected && typeof selected === "string") {
      setFolder(selected);
      setSelectedMount(undefined);
      setSelectedRoot(null);
    }
  }

  async function runScan(path: string) {
    setError(null);
    try {
      const snapshot = await startScan(path);
      applySnapshot(snapshot);
    } catch (err) {
      setError(formatUserError(err));
    }
  }

  async function handleStart() {
    if (!folder) {
      setError("Sélectionnez d'abord un dossier ou un disque ci-dessous.");
      return;
    }
    if (selectedRoot?.scope === "volume") {
      setVolumeConfirmOpen(true);
      return;
    }
    await runScan(folder);
  }

  function handlePickVolumeRoot(suggestion: ScanRootSuggestion) {
    setFolder(suggestion.path);
    setSelectedMount(suggestion.mountPath);
    setSelectedRoot(suggestion);
    setError(null);
  }

  useEffect(() => {
    if (consumedNavState.current) return;
    const state = location.state as { scanRoot?: ScanRootSuggestion } | null;
    const root = state?.scanRoot;
    if (!root) return;
    consumedNavState.current = true;
    handlePickVolumeRoot(root);
  }, [location.state]);

  async function handleCancel() {
    setError(null);
    try {
      const snapshot = await cancelScan();
      applySnapshot(snapshot);
    } catch (err) {
      setError(formatUserError(err));
    }
  }

  const isRunning = scan ? ACTIVE_STATUSES.has(scan.status) : false;
  const statusLabel = scan ? statusText(scan.status) : "Prêt";

  return (
    <>
      <PageHeader
        title="Analyse"
        description="Vue de tous les disques, puis parcours des fichiers indexés en base locale. Aucune modification de vos fichiers."
      />

      <VolumesOverview
        className="mb-6"
        deferLoadMs={500}
        onPickScanRoot={(s) => handlePickVolumeRoot(s)}
        selectedMountPath={selectedMount}
        description="Au démarrage, Giza analyse vos disques automatiquement. Ici, vous pouvez relancer ou cibler un dossier."
      />

      <div className="flex flex-wrap gap-3">
        <button
          type="button"
          onClick={pickFolder}
          disabled={isRunning}
          className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-elevated)] px-4 py-2 text-sm text-[var(--color-text)] hover:border-[var(--color-accent)] disabled:opacity-50"
        >
          Choisir un dossier
        </button>
        <button
          type="button"
          onClick={handleStart}
          disabled={!folder || isRunning}
          className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-muted)] disabled:opacity-50"
        >
          Démarrer l&apos;analyse
        </button>
        <button
          type="button"
          onClick={handleCancel}
          disabled={!isRunning}
          className="rounded-lg border border-red-500/40 px-4 py-2 text-sm text-red-300 hover:bg-red-500/10 disabled:opacity-50"
        >
          Annuler
        </button>
      </div>

      {folder ? (
        <p className="mt-4 break-all text-sm text-[var(--color-text-muted)]">
          Dossier : <span className="text-[var(--color-text)]">{folder}</span>
        </p>
      ) : null}

      {error ? (
        <InlineAlert variant="error" className="mt-3" title="Analyse interrompue">
          {error}
        </InlineAlert>
      ) : null}

      <section className="mt-8 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h3 className="text-sm font-medium text-[var(--color-text)]">Progression</h3>
          <ProgressStatus label={statusLabel} active={isRunning} />
        </div>

        {scan && scan.filesSeen > 0 ? (
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            {scan.filesSeen.toLocaleString("fr-FR")} fichiers — {formatBytes(scan.bytesTotal)}
          </p>
        ) : (
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Aucune analyse en cours ou pas encore démarrée.
          </p>
        )}

        {scan?.currentPath ? (
          <p className="mt-2 truncate font-mono text-xs text-[var(--color-text-muted)]">
            {scan.currentPath}
          </p>
        ) : null}

        {scan?.status === "completed" ? (
          <InlineAlert variant="success" className="mt-3" title="Analyse terminée">
            Scan #{scan.scanId} indexé. Les modules Doublons et Gros fichiers utiliseront ces données.
          </InlineAlert>
        ) : null}

        {scan?.status === "failed" && scan.errorMessage ? (
          <InlineAlert variant="error" className="mt-3" title="Échec de l'analyse">
            {scan.errorMessage}
          </InlineAlert>
        ) : null}

        {scan?.status === "cancelled" ? (
          <InlineAlert variant="info" className="mt-3">
            Analyse annulée. Vous pouvez en relancer une à tout moment.
          </InlineAlert>
        ) : null}
      </section>

      <ConfirmDialog
        open={volumeConfirmOpen}
        title="Analyser tout le volume ?"
        description={`L'analyse peut durer longtemps et solliciter fortement le disque : ${folder ?? ""}`}
        warning="Préférez le profil utilisateur si vous débutez. Vous pourrez annuler à tout moment."
        confirmLabel="Démarrer quand même"
        onConfirm={() => {
          setVolumeConfirmOpen(false);
          if (folder) void runScan(folder);
        }}
        onCancel={() => setVolumeConfirmOpen(false)}
      />
    </>
  );
}

function statusText(status: string): string {
  switch (status) {
    case "running":
      return "En cours";
    case "cancelling":
      return "Annulation…";
    case "completed":
      return "Terminé";
    case "cancelled":
      return "Annulé";
    case "failed":
      return "Échec";
    default:
      return "Prêt";
  }
}
