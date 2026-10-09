import { useCallback, useEffect, useMemo, useState } from "react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { useContextMenu } from "../../components/ContextMenu";
import { EmptyState } from "../../components/EmptyState";
import { InlineAlert } from "../../components/InlineAlert";
import { PageHeader } from "../../components/PageHeader";
import { ProgressStatus } from "../../components/ProgressStatus";
import { copyPathToClipboard, revealFileInExplorer, trashSingleFile } from "../../lib/fileActions";
import { formatUserError } from "../../lib/userError";
import {
  cancelDuplicateAnalysis,
  fetchDuplicateGroupDetails,
  fetchDuplicateGroups,
  fetchDuplicateMergePlan,
  getDuplicateStatus,
  moveFilesToTrash,
  onDuplicateFinished,
  onDuplicateProgress,
  startDuplicateAnalysis,
} from "../../lib/tauri";
import { formatBytes } from "../../lib/format";
import type {
  DuplicateGroup,
  DuplicateGroupsResult,
  DuplicateMember,
  DuplicateMergePlan,
  DuplicateSnapshot,
} from "../../types/dto";

const RUNNING = new Set(["running", "cancelling"]);

export function DuplicatesPage() {
  const { open: openMenu, menuPortal } = useContextMenu();
  const [analysis, setAnalysis] = useState<DuplicateSnapshot | null>(null);
  const [groups, setGroups] = useState<DuplicateGroupsResult | null>(null);
  const [expandedGroupId, setExpandedGroupId] = useState<number | null>(null);
  const [members, setMembers] = useState<DuplicateMember[]>([]);
  const [keepByGroup, setKeepByGroup] = useState<Record<number, string>>({});
  const [selectedPaths, setSelectedPaths] = useState<
    Record<string, { sizeBytes: number; groupId: number }>
  >({});
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [singleTrashPath, setSingleTrashPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<{ variant: "success" | "warning"; message: string } | null>(
    null,
  );
  const [loadingMembers, setLoadingMembers] = useState(false);
  const [trashing, setTrashing] = useState(false);
  const [mergeLoading, setMergeLoading] = useState(false);

  const refreshGroups = useCallback(async () => {
    try {
      const data = await fetchDuplicateGroups();
      setGroups(data);
    } catch (err) {
      setError(formatUserError(err));
    }
  }, []);

  useEffect(() => {
    getDuplicateStatus().then(setAnalysis).catch(() => setAnalysis(null));
    void refreshGroups();

    let unlistenProgress: (() => void) | undefined;
    let unlistenFinished: (() => void) | undefined;

    void (async () => {
      unlistenProgress = await onDuplicateProgress((payload) => {
        setAnalysis((prev) => ({
          scanId: payload.scanId,
          status: payload.status,
          candidatesHashed: payload.candidatesHashed,
          groupsFound: payload.groupsFound,
          recoverableBytes: payload.recoverableBytes,
          errorMessage: prev?.errorMessage,
        }));
      });
      unlistenFinished = await onDuplicateFinished((payload) => {
        setAnalysis({
          scanId: payload.scanId,
          status: payload.status,
          candidatesHashed: 0,
          groupsFound: payload.groupsFound,
          recoverableBytes: payload.recoverableBytes,
          errorMessage: payload.errorMessage,
        });
        void refreshGroups();
      });
    })();

    return () => {
      unlistenProgress?.();
      unlistenFinished?.();
    };
  }, [refreshGroups]);

  const selectedList = useMemo(() => Object.entries(selectedPaths), [selectedPaths]);
  const selectedBytes = useMemo(
    () => selectedList.reduce((sum, [, entry]) => sum + entry.sizeBytes, 0),
    [selectedList],
  );

  const groupsFullySelected = useMemo(() => {
    if (!groups?.groups.length) return [];
    const counts: Record<number, number> = {};
    for (const [, entry] of selectedList) {
      counts[entry.groupId] = (counts[entry.groupId] ?? 0) + 1;
    }
    return groups.groups.filter((g) => counts[g.id] === g.fileCount);
  }, [groups, selectedList]);

  const trashBlocked = groupsFullySelected.length > 0;

  function applyMergePlan(plan: DuplicateMergePlan, groupId?: number) {
    const keepUpdates: Record<number, string> = {};
    for (const item of plan.toRemove) {
      if (groupId != null && item.groupId !== groupId) continue;
      keepUpdates[item.groupId] = item.keepPath;
    }
    setKeepByGroup((k) => ({ ...k, ...keepUpdates }));

    setSelectedPaths((prev) => {
      const next = groupId == null ? {} : { ...prev };
      for (const item of plan.toRemove) {
        if (groupId != null && item.groupId !== groupId) continue;
        next[item.path] = { sizeBytes: item.sizeBytes, groupId: item.groupId };
      }
      return next;
    });
  }

  async function loadMergePlan(): Promise<DuplicateMergePlan | null> {
    setMergeLoading(true);
    try {
      return await fetchDuplicateMergePlan(groups?.scan?.id);
    } catch (err) {
      setError(formatUserError(err));
      return null;
    } finally {
      setMergeLoading(false);
    }
  }

  async function selectAllForMerge() {
    const plan = await loadMergePlan();
    if (!plan?.toRemove.length) {
      setNotice({ variant: "warning", message: "Aucun doublon à fusionner." });
      return;
    }
    applyMergePlan(plan);
    setNotice({
      variant: "success",
      message: `${plan.toRemove.length} copie(s) pré-sélectionnées — une copie par groupe sera conservée.`,
    });
  }

  async function mergeAllGroups() {
    const plan = await loadMergePlan();
    if (!plan?.toRemove.length) {
      setNotice({ variant: "warning", message: "Aucun doublon à fusionner." });
      return;
    }
    applyMergePlan(plan);
    setConfirmOpen(true);
  }

  async function mergeGroup(groupId: number) {
    const plan = await loadMergePlan();
    if (!plan) return;
    applyMergePlan(plan, groupId);
  }

  function clearSelection() {
    setSelectedPaths({});
  }

  async function handleAnalyze() {
    setError(null);
    try {
      const snapshot = await startDuplicateAnalysis();
      setAnalysis(snapshot);
    } catch (err) {
      setError(formatUserError(err));
    }
  }

  async function handleCancel() {
    setError(null);
    try {
      const snapshot = await cancelDuplicateAnalysis();
      setAnalysis(snapshot);
    } catch (err) {
      setError(formatUserError(err));
    }
  }

  async function toggleGroup(group: DuplicateGroup) {
    if (expandedGroupId === group.id) {
      setExpandedGroupId(null);
      setMembers([]);
      return;
    }
    setExpandedGroupId(group.id);
    setLoadingMembers(true);
    try {
      const [details, plan] = await Promise.all([
        fetchDuplicateGroupDetails(group.id),
        fetchDuplicateMergePlan(groups?.scan?.id),
      ]);
      const list = details?.members ?? [];
      setMembers(list);
      const plannedKeep = plan?.toRemove.find((item) => item.groupId === group.id)?.keepPath;
      const keepPath = plannedKeep ?? list[0]?.path;
      if (keepPath) {
        setKeepByGroup((prev) => ({ ...prev, [group.id]: keepPath }));
      }
    } catch (err) {
      setError(formatUserError(err));
      setMembers([]);
    } finally {
      setLoadingMembers(false);
    }
  }

  function togglePath(path: string, sizeBytes: number, groupId: number, checked: boolean) {
    setSelectedPaths((prev) => {
      const next = { ...prev };
      if (checked) next[path] = { sizeBytes, groupId };
      else delete next[path];
      return next;
    });
  }

  function openFileContextMenu(
    event: React.MouseEvent,
    member: DuplicateMember,
    groupId: number,
  ) {
    const isKeep = keepByGroup[groupId] === member.path;
    openMenu(
      event,
      [
        {
          id: "toggle-merge",
          label: member.path in selectedPaths ? "Décocher pour la fusion" : "Cocher pour la fusion",
          onSelect: () =>
            togglePath(
              member.path,
              member.sizeBytes,
              groupId,
              !(member.path in selectedPaths),
            ),
        },
        {
          id: "keep",
          label: "Conserver cette copie (décocher les autres)",
          disabled: isKeep,
          onSelect: () => void setAsKeepCopy(groupId, member),
        },
        {
          id: "reveal",
          label: "Afficher dans l'explorateur",
          onSelect: () => void revealFileInExplorer(member.path).catch(() => {}),
        },
        {
          id: "copy",
          label: "Copier le chemin",
          onSelect: () => void copyPathToClipboard(member.path),
        },
        {
          id: "trash",
          label: "Envoyer à la corbeille",
          danger: true,
          onSelect: () => setSingleTrashPath(member.path),
        },
      ],
      member.name,
    );
  }

  async function setAsKeepCopy(groupId: number, member: DuplicateMember) {
    setKeepByGroup((prev) => ({ ...prev, [groupId]: member.path }));
    setSelectedPaths((prev) => {
      const next = { ...prev };
      for (const m of members) {
        if (m.path === member.path) delete next[m.path];
        else next[m.path] = { sizeBytes: m.sizeBytes, groupId };
      }
      return next;
    });
  }

  async function confirmTrash() {
    setConfirmOpen(false);
    setTrashing(true);
    setError(null);
    setNotice(null);
    try {
      const result = await moveFilesToTrash(selectedList.map(([path]) => path));
      if (result.failedCount > 0) {
        setNotice({
          variant: "warning",
          message: `${result.successCount} envoyé(s), ${result.failedCount} échec(s). Consultez l'historique.`,
        });
      } else if (result.successCount > 0) {
        setNotice({
          variant: "success",
          message: `${result.successCount} fichier(s) fusionné(s) — copies en trop envoyées à la corbeille.`,
        });
      }
      setSelectedPaths({});
      if (expandedGroupId != null) {
        const details = await fetchDuplicateGroupDetails(expandedGroupId);
        setMembers(details?.members ?? []);
      }
      await refreshGroups();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setTrashing(false);
    }
  }

  async function confirmSingleTrash() {
    if (!singleTrashPath) return;
    const path = singleTrashPath;
    setSingleTrashPath(null);
    setTrashing(true);
    try {
      await trashSingleFile(path);
      setNotice({ variant: "success", message: "Fichier envoyé à la corbeille." });
      if (expandedGroupId != null) {
        const details = await fetchDuplicateGroupDetails(expandedGroupId);
        setMembers(details?.members ?? []);
      }
      await refreshGroups();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setTrashing(false);
    }
  }

  const isRunning = analysis ? RUNNING.has(analysis.status) : false;

  return (
    <>
      {menuPortal}
      <PageHeader
        title="Doublons"
        description="Fusion = corbeille pour les copies en trop, une copie conservée par groupe. Clic droit sur un fichier pour plus d'actions."
      />

      <div className="flex flex-wrap gap-3">
        <button type="button" onClick={handleAnalyze} disabled={isRunning} className="dp-btn-primary">
          Détecter les doublons
        </button>
        <button
          type="button"
          onClick={handleCancel}
          disabled={!isRunning}
          className="rounded-lg border border-red-500/40 px-4 py-2 text-sm text-red-300 hover:bg-red-500/10 disabled:opacity-50"
        >
          Annuler
        </button>
        <button type="button" onClick={() => refreshGroups()} className="dp-btn-secondary">
          Actualiser
        </button>
        <button
          type="button"
          disabled={!groups?.groups.length || mergeLoading}
          onClick={() => void selectAllForMerge()}
          className="dp-btn-secondary"
        >
          Tout cocher pour fusion
        </button>
        <button type="button" onClick={clearSelection} disabled={!selectedList.length} className="dp-btn-secondary">
          Tout décocher
        </button>
        <button
          type="button"
          disabled={!groups?.groups.length || mergeLoading}
          onClick={() => void mergeAllGroups()}
          className="dp-btn-secondary"
        >
          Fusionner tout…
        </button>
        <button
          type="button"
          disabled={selectedList.length === 0 || trashing}
          onClick={() => setConfirmOpen(true)}
          className="rounded-lg border border-amber-500/50 px-4 py-2 text-sm text-amber-200 hover:bg-amber-500/10 disabled:opacity-50"
        >
          Corbeille ({selectedList.length})
        </button>
      </div>

      {selectedList.length > 0 ? (
        <p className="mt-3 text-sm text-[var(--color-text-muted)]">
          Sélection fusion : {selectedList.length} fichier(s) — {formatBytes(selectedBytes)}
        </p>
      ) : null}

      {trashBlocked ? (
        <p className="mt-2 text-sm text-amber-300">
          Chaque groupe doit garder au moins une copie non cochée ({groupsFullySelected.length}{" "}
          groupe(s) entièrement cochés).
        </p>
      ) : null}

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

      <section className="mt-6 dp-card p-5 text-sm">
        {!groups?.scan ? (
          <EmptyState
            title="Aucune analyse indexée"
            description="Lancez d'abord une analyse de dossier, puis détectez les doublons sur ce scan."
            actionLabel="Aller à l'analyse"
            actionTo="/analyze"
          />
        ) : (
          <>
            <p className="text-[var(--color-text-muted)]">
              Scan #{groups.scan.id} — {groups.scan.rootPath}
            </p>
            <p className="mt-2 text-[var(--color-text)]">
              {groups.groupCount.toLocaleString("fr-FR")} groupe(s) —{" "}
              <span className="text-emerald-400">{formatBytes(groups.recoverableBytes)}</span>{" "}
              récupérables
            </p>
            {analysis && isRunning ? (
              <div className="mt-2">
                <ProgressStatus
                  label="Détection en cours"
                  detail={`${analysis.candidatesHashed.toLocaleString("fr-FR")} candidats hashés`}
                  active
                />
              </div>
            ) : null}
          </>
        )}
      </section>

      <ul className="mt-6 space-y-3">
        {groups?.groups.length ? (
          groups.groups.map((group) => (
            <li key={group.id} className="dp-card overflow-hidden">
              <div className="flex flex-wrap items-center gap-2 px-4 py-3">
                <button
                  type="button"
                  onClick={() => toggleGroup(group)}
                  className="min-w-0 flex-1 text-left"
                >
                  <p className="text-sm font-medium text-[var(--color-text)]">
                    {group.fileCount} fichiers — {formatBytes(group.totalBytes)}
                  </p>
                  <p className="mt-1 font-mono text-xs text-[var(--color-text-muted)]">
                    SHA-256 : {group.contentHash.slice(0, 16)}…
                  </p>
                </button>
                <div className="text-right text-sm text-emerald-400">
                  +{formatBytes(group.recoverableBytes)}
                </div>
                <button
                  type="button"
                  className="dp-btn-secondary text-xs"
                  onClick={() => void mergeGroup(group.id)}
                >
                  Fusionner ce groupe
                </button>
              </div>
              {expandedGroupId === group.id ? (
                <div className="border-t border-[var(--color-border)] px-4 py-3">
                  {loadingMembers ? (
                    <p className="text-sm text-[var(--color-text-muted)]">Chargement…</p>
                  ) : (
                    <ul className="space-y-2">
                      {members.map((member) => {
                        const isKeep = keepByGroup[group.id] === member.path;
                        return (
                          <li
                            key={member.fileId}
                            className="flex items-start gap-2 rounded-lg px-1 py-0.5 hover:bg-[var(--color-surface)]/50"
                            onContextMenu={(e) => openFileContextMenu(e, member, group.id)}
                          >
                            <input
                              type="checkbox"
                              className="mt-0.5"
                              checked={member.path in selectedPaths}
                              onChange={(e) =>
                                togglePath(
                                  member.path,
                                  member.sizeBytes,
                                  group.id,
                                  e.target.checked,
                                )
                              }
                            />
                            <span
                              className="min-w-0 flex-1 truncate font-mono text-xs text-[var(--color-text-muted)]"
                              title={member.path}
                            >
                              {isKeep ? (
                                <span className="mr-2 rounded bg-emerald-500/20 px-1.5 py-0.5 text-[10px] uppercase text-emerald-300">
                                  Conservée
                                </span>
                              ) : null}
                              {member.path} ({formatBytes(member.sizeBytes)})
                            </span>
                          </li>
                        );
                      })}
                    </ul>
                  )}
                  <p className="mt-3 text-xs text-[var(--color-text-muted)]">
                    Clic droit : cocher/décocher, conserver une copie, explorateur, corbeille.
                  </p>
                </div>
              ) : null}
            </li>
          ))
        ) : groups?.scan ? (
          <li>
            <EmptyState
              title="Aucun doublon détecté"
              description="Lancez « Détecter les doublons » ou attendez la fin de l'analyse automatique."
            />
          </li>
        ) : null}
      </ul>

      <ConfirmDialog
        open={confirmOpen}
        title="Fusionner les doublons ?"
        description={`${selectedList.length} copie(s) seront envoyées à la Corbeille (${formatBytes(selectedBytes)}). Une copie par groupe reste sur le disque.`}
        warning={
          trashBlocked
            ? "Chaque groupe doit conserver au moins une copie non sélectionnée."
            : undefined
        }
        confirmDisabled={trashBlocked}
        confirmLabel="Fusionner (corbeille)"
        onConfirm={() => void confirmTrash()}
        onCancel={() => setConfirmOpen(false)}
      />

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
