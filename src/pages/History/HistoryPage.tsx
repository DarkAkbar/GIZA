import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { EmptyState } from "../../components/EmptyState";
import { InlineAlert } from "../../components/InlineAlert";
import { LoadingBlock } from "../../components/LoadingBlock";
import { PageHeader } from "../../components/PageHeader";
import { copyPathToClipboard, revealFileInExplorer } from "../../lib/fileActions";
import { OPERATIONS_REFRESH_EVENT } from "../../lib/operationsRefresh";
import { fetchOperations } from "../../lib/tauri";
import { formatBytes, formatIsoDate } from "../../lib/format";
import { formatUserError } from "../../lib/userError";
import type { MoveItemOutcome, OperationRecord, TrashItemOutcome } from "../../types/dto";

type FilterId = "all" | "trash" | "organize" | "uninstall" | "other";

const FILTERS: { id: FilterId; label: string }[] = [
  { id: "all", label: "Tous" },
  { id: "trash", label: "Corbeille" },
  { id: "organize", label: "Organisation" },
  { id: "uninstall", label: "Désinstallation" },
  { id: "other", label: "Autres" },
];

export function HistoryPage() {
  const [operations, setOperations] = useState<OperationRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterId>("all");
  const [expandedId, setExpandedId] = useState<number | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setOperations(await fetchOperations(200));
    } catch (err) {
      setError(formatUserError(err));
      setOperations([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const onRefresh = () => void load();
    window.addEventListener(OPERATIONS_REFRESH_EVENT, onRefresh);
    return () => window.removeEventListener(OPERATIONS_REFRESH_EVENT, onRefresh);
  }, [load]);

  const filtered = useMemo(() => {
    if (filter === "all") return operations;
    return operations.filter((op) => filterMatches(op, filter));
  }, [operations, filter]);

  const summary = useMemo(() => {
    let trashFiles = 0;
    let trashBytes = 0;
    let uninstalls = 0;
    for (const op of operations) {
      if (op.operationType === "trash") {
        const parsed = parseTrashPayload(op.payloadJson);
        trashFiles += parsed.ok;
        trashBytes += op.recoverableBytes ?? 0;
      }
      if (op.operationType === "uninstall") uninstalls += 1;
    }
    return { trashFiles, trashBytes, uninstalls };
  }, [operations]);

  return (
    <>
      <PageHeader
        title="Historique"
        description="Journal local : corbeille, déplacements, lancement de désinstallation Windows. Les fichiers en corbeille restent récupérables via Windows."
      />

      {operations.length > 0 ? (
        <div className="mb-4 flex flex-wrap gap-4 text-sm text-[var(--color-text-muted)]">
          <span>
            <strong className="text-[var(--color-text)]">
              {summary.trashFiles.toLocaleString("fr-FR")}
            </strong>{" "}
            fichier(s) en corbeille via Giza
            {summary.trashBytes > 0 ? ` (${formatBytes(summary.trashBytes)})` : null}
          </span>
          {summary.uninstalls > 0 ? (
            <span>
              <strong className="text-[var(--color-text)]">{summary.uninstalls}</strong> assistant(s)
              de désinstallation lancé(s)
            </span>
          ) : null}
          <Link to="/cleanup" className="text-orange-400 hover:underline">
            Nettoyage
          </Link>
          <Link to="/duplicates" className="text-orange-400 hover:underline">
            Doublons
          </Link>
        </div>
      ) : null}

      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">
          {FILTERS.map((f) => (
            <button
              key={f.id}
              type="button"
              onClick={() => setFilter(f.id)}
              className={[
                "rounded-lg px-3 py-1.5 text-sm transition-colors",
                filter === f.id
                  ? "bg-[var(--color-accent)] text-white"
                  : "border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)]",
              ].join(" ")}
            >
              {f.label}
            </button>
          ))}
        </div>
        <button
          type="button"
          onClick={() => void load()}
          disabled={loading}
          className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
        >
          Actualiser
        </button>
      </div>

      {error ? (
        <InlineAlert variant="error" title="Impossible de charger l'historique" onRetry={() => void load()}>
          {error}
        </InlineAlert>
      ) : null}

      {loading && operations.length === 0 ? (
        <LoadingBlock label="Chargement du journal…" className="mt-4" />
      ) : !error && filtered.length === 0 ? (
        <EmptyState
          className="mt-4"
          title={filter === "all" ? "Aucune opération enregistrée" : "Aucune entrée pour ce filtre"}
          description="Les envois à la corbeille, déplacements et désinstallations lancées depuis Giza apparaîtront ici."
        />
      ) : (
        <ul className="mt-4 space-y-2">
          {filtered.map((op) => (
            <OperationCard
              key={op.id}
              op={op}
              expanded={expandedId === op.id}
              onToggle={() => setExpandedId((id) => (id === op.id ? null : op.id))}
            />
          ))}
        </ul>
      )}
    </>
  );
}

function filterMatches(op: OperationRecord, filter: FilterId): boolean {
  switch (filter) {
    case "trash":
      return op.operationType === "trash";
    case "organize":
      return op.operationType === "organize" || op.operationType === "move";
    case "uninstall":
      return op.operationType === "uninstall";
    case "other":
      return !["trash", "organize", "move", "uninstall"].includes(op.operationType);
    default:
      return true;
  }
}

function OperationCard({
  op,
  expanded,
  onToggle,
}: {
  op: OperationRecord;
  expanded: boolean;
  onToggle: () => void;
}) {
  const summary = summarizeOperation(op);
  const hasDetail = summary.detailLines.length > 0;

  return (
    <li className="overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)]">
      <button
        type="button"
        className="flex w-full flex-wrap items-start gap-3 px-4 py-3 text-left hover:bg-[var(--color-surface)]/40"
        onClick={() => hasDetail && onToggle()}
        disabled={!hasDetail}
      >
        <div className="min-w-[8rem] text-xs text-[var(--color-text-muted)]">
          {formatIsoDate(op.createdAt)}
        </div>
        <div className="min-w-[7rem]">
          <span
            className={[
              "inline-block rounded-md px-2 py-0.5 text-xs font-medium",
              badgeClass(op.operationType),
            ].join(" ")}
          >
            {labelForType(op.operationType)}
          </span>
          {op.reversible ? (
            <span className="ml-2 text-[10px] uppercase text-emerald-400/80">Réversible</span>
          ) : (
            <span className="ml-2 text-[10px] uppercase text-[var(--color-text-muted)]">
              Assisté Windows
            </span>
          )}
        </div>
        <div className="min-w-[5rem] text-sm text-[var(--color-text)]">
          {op.recoverableBytes != null && op.recoverableBytes > 0
            ? formatBytes(op.recoverableBytes)
            : "—"}
        </div>
        <div className="min-w-0 flex-1 text-sm text-[var(--color-text-muted)]">
          {summary.headline}
          {hasDetail ? (
            <span className="ml-2 text-xs text-orange-400/90">
              {expanded ? "Masquer" : "Détail"}
            </span>
          ) : null}
        </div>
      </button>
      {expanded && hasDetail ? (
        <div className="border-t border-[var(--color-border)] px-4 py-3">
          <ul className="max-h-64 space-y-2 overflow-y-auto text-xs">
            {summary.detailLines.map((line) => (
              <li
                key={line.key}
                className="flex flex-wrap items-center gap-2 rounded-lg bg-[var(--color-surface)]/60 px-2 py-1.5 font-mono"
              >
                <span
                  className={[
                    "shrink-0",
                    line.success === false ? "text-red-400" : "text-[var(--color-text-muted)]",
                  ].join(" ")}
                  title={line.text}
                >
                  {line.success === false ? "Échec" : "OK"} — {line.text}
                </span>
                {line.path ? (
                  <span className="ml-auto flex gap-2">
                    <button
                      type="button"
                      className="text-orange-400 hover:underline"
                      onClick={() => void copyPathToClipboard(line.path!)}
                    >
                      Copier
                    </button>
                    <button
                      type="button"
                      className="text-orange-400 hover:underline"
                      onClick={() => void revealFileInExplorer(line.path!).catch(() => {})}
                    >
                      Explorateur
                    </button>
                  </span>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </li>
  );
}

type DetailLine = {
  key: string;
  text: string;
  path?: string;
  success?: boolean;
};

function summarizeOperation(op: OperationRecord): { headline: string; detailLines: DetailLine[] } {
  if (op.operationType === "trash") {
    const parsed = parseTrashPayload(op.payloadJson);
    return {
      headline: `${parsed.ok} fichier(s) envoyé(s) à la corbeille${parsed.fail > 0 ? `, ${parsed.fail} échec(s)` : ""}`,
      detailLines: parsed.outcomes.map((o, i) => ({
        key: `${op.id}-${i}-${o.path}`,
        text: o.success
          ? `${o.path} (${formatBytes(o.sizeBytes)})`
          : `${o.path} — ${o.error ?? "erreur"}`,
        path: o.path,
        success: o.success,
      })),
    };
  }
  if (op.operationType === "organize" || op.operationType === "move") {
    const outcomes = parseMoveOutcomes(op.payloadJson);
    const ok = outcomes.filter((o) => o.success).length;
    return {
      headline: `${ok} déplacement(s) réussi(s)${outcomes.length - ok > 0 ? `, ${outcomes.length - ok} échec(s)` : ""}`,
      detailLines: outcomes.map((o, i) => ({
        key: `${op.id}-${i}`,
        text: o.success
          ? `${o.sourcePath} → ${o.destPath}`
          : `${o.sourcePath} — ${o.error ?? "erreur"}`,
        path: o.sourcePath,
        success: o.success,
      })),
    };
  }
  if (op.operationType === "uninstall") {
    const payload = parseUninstallPayload(op.payloadJson);
    return {
      headline: `Assistant Windows lancé pour « ${payload.appName} »${payload.publisher ? ` (${payload.publisher})` : ""}`,
      detailLines: [
        {
          key: `${op.id}-0`,
          text: "Giza n'a pas supprimé l'application — suivez les étapes de l'assistant système.",
        },
      ],
    };
  }
  return {
    headline: op.payloadJson.slice(0, 160) || op.operationType,
    detailLines: [],
  };
}

function parseTrashPayload(json: string) {
  try {
    const parsed = JSON.parse(json) as { outcomes?: TrashItemOutcome[] };
    const outcomes = parsed.outcomes ?? [];
    const ok = outcomes.filter((o) => o.success).length;
    return { outcomes, ok, fail: outcomes.length - ok };
  } catch {
    return { outcomes: [] as TrashItemOutcome[], ok: 0, fail: 0 };
  }
}

function parseMoveOutcomes(json: string): MoveItemOutcome[] {
  try {
    return JSON.parse(json) as MoveItemOutcome[];
  } catch {
    return [];
  }
}

function parseUninstallPayload(json: string): { appName: string; publisher?: string } {
  try {
    const p = JSON.parse(json) as { appName?: string; app_name?: string; publisher?: string };
    return {
      appName: p.appName ?? p.app_name ?? "Application",
      publisher: p.publisher,
    };
  } catch {
    return { appName: "Application" };
  }
}

function labelForType(type: string): string {
  switch (type) {
    case "trash":
      return "Corbeille";
    case "move":
      return "Déplacement";
    case "organize":
      return "Organisation";
    case "uninstall":
      return "Désinstallation";
    case "scan":
      return "Analyse";
    default:
      return type;
  }
}

function badgeClass(type: string): string {
  switch (type) {
    case "trash":
      return "bg-amber-500/20 text-amber-200";
    case "organize":
    case "move":
      return "bg-sky-500/20 text-sky-200";
    case "uninstall":
      return "bg-violet-500/20 text-violet-200";
    default:
      return "bg-[var(--color-border)] text-[var(--color-text-muted)]";
  }
}
