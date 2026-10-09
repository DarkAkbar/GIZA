import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { ProgressStatus } from "./ProgressStatus";
import { wasBootstrapStartedInSession } from "./SplashGate";
import {
  ANALYSIS_REFRESH_EVENT,
  ensureBootstrapAnalysis,
  fetchBootstrapStatus,
  getScanStatus,
} from "../lib/tauri";
import type { BootstrapStatus } from "../types/dto";

function dispatchRefresh() {
  window.dispatchEvent(new CustomEvent(ANALYSIS_REFRESH_EVENT));
}

type VolumeReadyHint = "scan" | "duplicates" | null;

const QUICK_LINKS = [
  { to: "/dashboard", label: "Dashboard" },
  { to: "/large-files", label: "Gros fichiers" },
  { to: "/duplicates", label: "Doublons" },
  { to: "/cleanup", label: "Nettoyage" },
] as const;

export function BootstrapStatusBanner() {
  const [status, setStatus] = useState<BootstrapStatus | null>(null);
  const [volumeReady, setVolumeReady] = useState<VolumeReadyHint>(null);

  const refreshStatus = useCallback(async () => {
    try {
      setStatus(await fetchBootstrapStatus());
    } catch {
      setStatus(null);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const scan = await getScanStatus();
        if (scan.status === "running" || scan.status === "cancelling") {
          if (!cancelled) await refreshStatus();
          return;
        }
        if (!wasBootstrapStartedInSession()) {
          const result = await ensureBootstrapAnalysis();
          if (!cancelled) setStatus(result);
          dispatchRefresh();
          return;
        }
        if (!cancelled) await refreshStatus();
      } catch {
        if (!cancelled) await refreshStatus();
      }
    })();

    const unsubs: (() => void)[] = [];
    void (async () => {
      unsubs.push(
        await listen<{ scanId: number; status: string }>("scan-finished", (event) => {
          if (event.payload.status === "completed") {
            setVolumeReady("scan");
          }
          void refreshStatus();
          dispatchRefresh();
        }),
      );
      unsubs.push(
        await listen<{ status: string }>("duplicate-finished", (event) => {
          if (event.payload.status === "completed") {
            setVolumeReady("duplicates");
          }
          void refreshStatus();
          dispatchRefresh();
        }),
      );
      unsubs.push(
        await listen("bootstrap-complete", () => {
          setVolumeReady(null);
          void refreshStatus();
          dispatchRefresh();
        }),
      );
    })();

    return () => {
      cancelled = true;
      unsubs.forEach((fn) => fn());
    };
  }, [refreshStatus]);

  if (!status || status.phase === "done" || status.phase === "idle") {
    return null;
  }

  const active = status.phase === "scanning" || status.phase === "duplicates";

  return (
    <div className="mb-4 rounded-xl border border-orange-500/30 bg-orange-500/10 px-4 py-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-sm font-medium text-[var(--color-text)]">Analyse automatique</p>
        <ProgressStatus
          label={status.phase === "duplicates" ? "Doublons" : "Indexation"}
          active={active}
        />
      </div>
      <p className="mt-1 text-sm text-[var(--color-text-muted)]">{status.message}</p>
      {status.volumesRemaining > 0 ? (
        <p className="mt-1 text-xs text-[var(--color-text-muted)]">
          Volumes restants : {status.volumesRemaining}
          {status.currentRoot ? ` — ${status.currentRoot.replace(/\\$/, "")}` : null}
        </p>
      ) : null}

      {volumeReady === "scan" ? (
        <p className="mt-2 text-sm text-[var(--color-text)]">
          Indexation terminée pour ce volume —{" "}
          <Link to="/large-files" className="font-medium text-orange-400 underline-offset-2 hover:underline">
            voir les gros fichiers
          </Link>
          {" · "}
          <Link to="/dashboard" className="font-medium text-orange-400 underline-offset-2 hover:underline">
            dashboard
          </Link>
        </p>
      ) : null}

      {volumeReady === "duplicates" ? (
        <p className="mt-2 text-sm text-[var(--color-text)]">
          Doublons à jour pour ce volume —{" "}
          <Link to="/duplicates" className="font-medium text-orange-400 underline-offset-2 hover:underline">
            ouvrir les doublons
          </Link>
        </p>
      ) : null}

      <div className="mt-3 flex flex-wrap gap-x-3 gap-y-1 text-xs">
        <span className="text-[var(--color-text-muted)]">Accès rapide :</span>
        {QUICK_LINKS.map((link) => (
          <Link
            key={link.to}
            to={link.to}
            className="text-orange-400/90 underline-offset-2 hover:text-orange-300 hover:underline"
          >
            {link.label}
          </Link>
        ))}
      </div>

      <p className="mt-2 text-xs text-[var(--color-text-muted)]">
        Giza enchaîne indexation et doublons volume par volume pour limiter la charge.
      </p>
    </div>
  );
}
