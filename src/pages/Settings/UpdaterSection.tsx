import { useEffect, useState } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { InlineAlert } from "../../components/InlineAlert";
import { fetchAppInfo } from "../../lib/tauri";
import { formatUserError } from "../../lib/userError";

export function UpdaterSection() {
  const [version, setVersion] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function loadVersion() {
    try {
      const info = await fetchAppInfo();
      setVersion(`${info.name} ${info.version}`);
    } catch {
      setVersion(null);
    }
  }

  useEffect(() => {
    void loadVersion();
  }, []);

  async function checkForUpdates() {
    setChecking(true);
    setError(null);
    setMessage(null);
    try {
      const update = await check();
      if (!update) {
        setMessage("Vous utilisez déjà la dernière version disponible.");
        return;
      }
      setMessage(
        `Mise à jour ${update.version} disponible${update.body ? ` — ${update.body}` : ""}.`,
      );
      setInstalling(true);
      await update.downloadAndInstall();
      setMessage("Installation terminée. Redémarrage…");
      await relaunch();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setChecking(false);
      setInstalling(false);
    }
  }

  return (
    <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
      <h2 className="text-sm font-medium text-[var(--color-text)]">Mises à jour</h2>
      <p className="mt-2 text-sm text-[var(--color-text-muted)]">
        Vérification via le flux configuré dans l&apos;installeur (GitHub Releases ou serveur
        HTTPS). Aucune mise à jour n&apos;est installée sans cette action.
      </p>
      <p className="mt-2 text-xs text-[var(--color-text-muted)]">
        Version installée : <strong>{version ?? "…"}</strong>
      </p>
      {error ? (
        <InlineAlert variant="error" className="mt-3">
          {error}
        </InlineAlert>
      ) : null}
      {message ? (
        <InlineAlert variant="success" className="mt-3">
          {message}
        </InlineAlert>
      ) : null}
      <button
        type="button"
        onClick={() => void checkForUpdates()}
        disabled={checking || installing}
        className="mt-4 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
      >
        {checking || installing ? "Vérification…" : "Rechercher les mises à jour"}
      </button>
    </section>
  );
}
