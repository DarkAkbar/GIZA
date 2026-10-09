import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  addUserExclusion,
  fetchExclusions,
  removeUserExclusion,
  resetUserExclusions,
} from "../../lib/tauri";
import { InlineAlert } from "../../components/InlineAlert";
import { formatUserError } from "../../lib/userError";
import type { ExclusionsOverview } from "../../types/dto";

export function ExclusionsSection() {
  const [overview, setOverview] = useState<ExclusionsOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setOverview(await fetchExclusions());
    } catch (err) {
      setError(formatUserError(err));
      setOverview(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function handleAddFolder() {
    setError(null);
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choisir un dossier à exclure de l'analyse",
    });
    if (selected == null || Array.isArray(selected)) return;

    setBusy(true);
    try {
      await addUserExclusion(selected);
      await load();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleRemove(id: number) {
    setBusy(true);
    setError(null);
    try {
      await removeUserExclusion(id);
      await load();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleReset() {
    if (!overview?.user.length) return;
    setBusy(true);
    setError(null);
    try {
      await resetUserExclusions();
      await load();
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
      <h2 className="text-sm font-medium text-[var(--color-text)]">Dossiers exclus de l&apos;analyse</h2>
      <p className="mt-2 text-sm text-[var(--color-text-muted)]">
        Les analyses ne parcourent pas ces dossiers (en plus des zones système protégées).
      </p>

      {error ? (
        <InlineAlert variant="error" className="mt-3" onRetry={() => void load()}>
          {error}
        </InlineAlert>
      ) : null}

      <div className="mt-5">
        <h3 className="text-xs uppercase tracking-wide text-[var(--color-text-muted)]">
          Dossiers système protégés
        </h3>
        <ul className="mt-2 space-y-1 text-sm text-[var(--color-text-muted)]">
          {loading && !overview ? (
            <li>Chargement…</li>
          ) : (
            overview?.system.map((item) => (
              <li key={item.name} className="flex items-center justify-between gap-2 rounded-lg border border-[var(--color-border)]/60 px-3 py-2">
                <span className="font-mono text-xs">{item.name}</span>
                <span className="text-[10px] uppercase text-emerald-400/90">Protégé par Giza</span>
              </li>
            ))
          )}
        </ul>
      </div>

      <div className="mt-6">
        <h3 className="text-xs uppercase tracking-wide text-[var(--color-text-muted)]">
          Exclusions personnelles
        </h3>
        <div className="mt-3 flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => void handleAddFolder()}
            disabled={busy || loading}
            className="rounded-lg bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white disabled:opacity-50"
          >
            + Ajouter un dossier
          </button>
          <button
            type="button"
            onClick={() => void handleReset()}
            disabled={busy || loading || !overview?.user.length}
            className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] disabled:opacity-50"
          >
            Réinitialiser
          </button>
        </div>
        <ul className="mt-4 space-y-2">
          {!loading && overview?.user.length === 0 ? (
            <li className="text-sm text-[var(--color-text-muted)]">
              Aucune exclusion personnelle. Les analyses utilisent uniquement les protections système.
            </li>
          ) : null}
          {overview?.user.map((row) => (
            <li
              key={row.id}
              className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-[var(--color-border)] px-3 py-2"
            >
              <span className="break-all font-mono text-xs text-[var(--color-text)]">{row.path}</span>
              <button
                type="button"
                disabled={busy}
                onClick={() => void handleRemove(row.id)}
                className="shrink-0 text-sm text-red-300 hover:text-red-200 disabled:opacity-50"
              >
                Supprimer
              </button>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
