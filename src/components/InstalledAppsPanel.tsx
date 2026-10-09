import { useCallback, useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AppIcon } from "./AppIcon";
import { ConfirmDialog } from "./ConfirmDialog";
import { useContextMenu } from "./ContextMenu";
import { InlineAlert } from "./InlineAlert";
import { LoadingBlock } from "./LoadingBlock";
import { fetchInstalledApps, uninstallApplication } from "../lib/tauri";
import { formatBytes } from "../lib/format";
import type { InstalledApp } from "../types/dto";

type InstalledAppsPanelProps = {
  limit?: number;
  compact?: boolean;
  className?: string;
};

export function InstalledAppsPanel({
  limit = 12,
  compact = false,
  className = "",
}: InstalledAppsPanelProps) {
  const { open: openMenu, menuPortal } = useContextMenu();
  const [apps, setApps] = useState<InstalledApp[]>([]);
  const [supported, setSupported] = useState(true);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uninstallTarget, setUninstallTarget] = useState<InstalledApp | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await fetchInstalledApps(limit);
      setApps(result.apps);
      setSupported(result.platformSupported);
    } catch {
      setError("Impossible de lire la liste des applications.");
      setApps([]);
    } finally {
      setLoading(false);
    }
  }, [limit]);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(), compact ? 400 : 0);
    return () => window.clearTimeout(timer);
  }, [compact, load]);

  async function openAppsSettings() {
    try {
      await openUrl("ms-settings:appsfeatures");
    } catch {
      /* hors Tauri */
    }
  }

  async function confirmUninstall() {
    if (!uninstallTarget) return;
    const target = uninstallTarget;
    setUninstallTarget(null);
    try {
      await uninstallApplication(target.name, {
        publisher: target.publisher,
        estimatedSizeBytes: target.estimatedSizeBytes,
      });
    } catch {
      setError(`Impossible de lancer la désinstallation de « ${target.name} ».`);
    }
  }

  function openAppMenu(event: React.MouseEvent, app: InstalledApp) {
    openMenu(
      event,
      [
        {
          id: "settings-apps",
          label: "Ouvrir Applications (Windows)",
          onSelect: () => void openAppsSettings(),
        },
        {
          id: "uninstall",
          label: "Désinstaller ce programme…",
          danger: true,
          disabled: !app.canUninstall,
          onSelect: () => setUninstallTarget(app),
        },
      ],
      app.name,
    );
  }

  return (
    <section className={`dp-card p-5 ${className}`.trim()}>
      {menuPortal}
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h2 className="text-sm font-medium text-[var(--color-text)]">
            Applications installées
          </h2>
          <p className="mt-1 text-xs text-[var(--color-text-muted)]">
            Clic droit sur une app : désinstallation Windows (assistée). Giza ne supprime jamais
            seul.
          </p>
        </div>
        <button type="button" className="dp-btn-ghost text-xs" onClick={() => void load()}>
          Actualiser
        </button>
      </div>

      {loading ? (
        <LoadingBlock label="Lecture des applications…" className="mt-4" />
      ) : error ? (
        <InlineAlert variant="error" className="mt-4" onRetry={() => void load()}>
          {error}
        </InlineAlert>
      ) : !supported ? (
        <InlineAlert variant="info" className="mt-4">
          Liste des applications disponible sur Windows uniquement.
        </InlineAlert>
      ) : apps.length === 0 ? (
        <InlineAlert variant="info" className="mt-4">
          Aucune application détectée via le registre.
        </InlineAlert>
      ) : (
        <ul
          className={[
            "mt-4 grid gap-2",
            compact ? "sm:grid-cols-2" : "sm:grid-cols-2 lg:grid-cols-3",
          ].join(" ")}
        >
          {apps.map((app) => (
            <li key={app.name}>
              <AppRow app={app} onContextMenu={(e) => openAppMenu(e, app)} />
            </li>
          ))}
        </ul>
      )}

      <button type="button" onClick={() => void openAppsSettings()} className="dp-btn-secondary mt-4">
        Ouvrir Applications (Windows)
      </button>

      <ConfirmDialog
        open={uninstallTarget != null}
        title="Lancer la désinstallation ?"
        description={
          uninstallTarget
            ? `Windows va ouvrir l'assistant de désinstallation pour « ${uninstallTarget.name} ».`
            : ""
        }
        warning="Vérifiez le nom du programme avant de confirmer dans l'assistant Windows."
        confirmLabel="Lancer la désinstallation"
        onConfirm={() => void confirmUninstall()}
        onCancel={() => setUninstallTarget(null)}
      />
    </section>
  );
}

function AppRow({
  app,
  onContextMenu,
}: {
  app: InstalledApp;
  onContextMenu: (e: React.MouseEvent) => void;
}) {
  return (
    <div
      className="flex cursor-context-menu items-center gap-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]/80 px-3 py-2.5 transition hover:border-[var(--color-accent)]/35"
      onContextMenu={onContextMenu}
    >
      <AppIcon name={app.name} iconDataUrl={app.iconDataUrl} size="sm" />
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-[var(--color-text)]">{app.name}</p>
        <p className="truncate text-xs text-[var(--color-text-muted)]">
          {app.publisher ?? "Éditeur inconnu"}
          {app.estimatedSizeBytes
            ? ` · ${formatBytes(app.estimatedSizeBytes)}`
            : " · taille inconnue"}
        </p>
      </div>
    </div>
  );
}
