import { useCallback, useEffect, useState } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchSettings, saveSettings } from "../../lib/tauri";
import { LARGE_FILE_PRESETS_MB } from "../../lib/format";
import { formatUserError } from "../../lib/userError";
import { InlineAlert } from "../../components/InlineAlert";
import { LoadingBlock } from "../../components/LoadingBlock";
import type { AppSettings, DuplicateKeepRule, SettingsResponse } from "../../types/dto";

const DUPLICATE_KEEP_OPTIONS: { id: DuplicateKeepRule; label: string; hint: string }[] = [
  {
    id: "shortestPath",
    label: "Chemin le plus court",
    hint: "Conserve le fichier dont le chemin complet est le plus court (départage en cas d'égalité).",
  },
  {
    id: "newest",
    label: "Plus récent",
    hint: "Conserve la copie avec la date de modification la plus récente.",
  },
  {
    id: "oldest",
    label: "Plus ancien",
    hint: "Conserve la copie la plus ancienne (souvent l'original).",
  },
];
import { ExclusionsSection } from "./ExclusionsSection";
import { UpdaterSection } from "./UpdaterSection";

export function SettingsPage() {
  const [data, setData] = useState<SettingsResponse | null>(null);
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [customMb, setCustomMb] = useState("");
  const [customReminderDays, setCustomReminderDays] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchSettings();
      setData(response);
      setDraft(response.settings);
      setCustomMb("");
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function handleSave() {
    if (!draft || !data) return;
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      const response = await saveSettings({
        largeFileThresholdMb: draft.largeFileThresholdMb,
        confirmSensitiveActions: draft.confirmSensitiveActions,
        notificationsEnabled: draft.notificationsEnabled,
        launchAtStartup: draft.launchAtStartup,
        duplicateKeepRule: draft.duplicateKeepRule,
        analysisReminderEnabled: draft.analysisReminderEnabled,
        analysisReminderIntervalDays: draft.analysisReminderIntervalDays,
      });
      setData(response);
      setDraft(response.settings);
      setSaved(true);
    } catch (err) {
      setError(formatUserError(err));
    } finally {
      setSaving(false);
    }
  }

  function applyCustomThreshold() {
    const mb = Number(customMb.replace(",", "."));
    if (!Number.isFinite(mb) || mb <= 0) {
      setError("Seuil personnalisé invalide : entrez un nombre de Mo supérieur à 0.");
      return;
    }
    setError(null);
    setDraft((prev) => (prev ? { ...prev, largeFileThresholdMb: Math.floor(mb) } : prev));
  }

  if (loading && !draft) {
    return (
      <>
        <PageHeader title="Paramètres" description="Chargement…" />
        <LoadingBlock label="Chargement des réglages…" />
      </>
    );
  }

  if (!draft || !data) {
    return (
      <>
        <PageHeader title="Paramètres" description="Impossible de charger les réglages." />
        {error ? (
          <InlineAlert variant="error" onRetry={() => void load()}>
            {error}
          </InlineAlert>
        ) : null}
      </>
    );
  }

  return (
    <>
      <PageHeader
        title="Paramètres"
        description="Préférences enregistrées localement dans SQLite. Aucune donnée n'est envoyée en ligne."
      />

      {error ? (
        <InlineAlert variant="error" className="mb-4">
          {error}
        </InlineAlert>
      ) : null}
      {saved ? (
        <InlineAlert variant="success" className="mb-4">
          Paramètres enregistrés.
        </InlineAlert>
      ) : null}

      <div className="space-y-8">
        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
          <h2 className="text-sm font-medium text-[var(--color-text)]">Planification de l&apos;analyse</h2>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Rappel discret lorsque la dernière indexation est ancienne. Aucune suppression ni scan
            automatique sans votre action.
          </p>
          <ul className="mt-4 space-y-4 text-sm">
            <SettingToggle
              label="Rappel de ré-analyse"
              hint="Bandeau sur le dashboard et notification Windows (si les notifications sont activées)."
              checked={draft.analysisReminderEnabled}
              onChange={(v) => setDraft({ ...draft, analysisReminderEnabled: v })}
            />
          </ul>
          <div className="mt-4 flex flex-wrap gap-2">
            {[7, 14, 30].map((days) => (
              <button
                key={days}
                type="button"
                disabled={!draft.analysisReminderEnabled}
                onClick={() => setDraft({ ...draft, analysisReminderIntervalDays: days })}
                className={[
                  "rounded-lg px-3 py-1.5 text-sm transition-colors disabled:opacity-40",
                  draft.analysisReminderIntervalDays === days
                    ? "bg-[var(--color-accent)] text-white"
                    : "border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)]",
                ].join(" ")}
              >
                {days} jours
              </button>
            ))}
          </div>
          <div className="mt-4 flex flex-wrap items-end gap-2">
            <label className="flex flex-col gap-1 text-xs text-[var(--color-text-muted)]">
              Intervalle personnalisé (jours)
              <input
                type="number"
                min={1}
                max={90}
                disabled={!draft.analysisReminderEnabled}
                value={customReminderDays}
                onChange={(e) => setCustomReminderDays(e.target.value)}
                className="w-36 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-text)] disabled:opacity-40"
                placeholder="ex. 21"
              />
            </label>
            <button
              type="button"
              disabled={!draft.analysisReminderEnabled}
              onClick={() => {
                const days = Number(customReminderDays.replace(",", "."));
                if (!Number.isFinite(days) || days < 1 || days > 90) {
                  setError("Intervalle invalide : entre 1 et 90 jours.");
                  return;
                }
                setError(null);
                setDraft({ ...draft, analysisReminderIntervalDays: Math.floor(days) });
              }}
              className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text)] hover:border-[var(--color-accent)] disabled:opacity-40"
            >
              Appliquer au brouillon
            </button>
          </div>
          <p className="mt-3 text-xs text-[var(--color-text-muted)]">
            Intervalle actuel :{" "}
            <strong>{draft.analysisReminderIntervalDays.toLocaleString("fr-FR")} jours</strong>
          </p>
        </section>

        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
          <h2 className="text-sm font-medium text-[var(--color-text)]">Général</h2>
          <ul className="mt-4 space-y-4 text-sm">
            <SettingToggle
              label="Confirmer les actions sensibles"
              hint="Corbeille, déplacements — une fenêtre de confirmation s'affichera (recommandé)."
              checked={draft.confirmSensitiveActions}
              onChange={(v) => setDraft({ ...draft, confirmSensitiveActions: v })}
            />
            <SettingToggle
              label="Afficher les notifications"
              hint="Alertes Windows pour les recommandations importantes et la fin d'analyse (sans spam)."
              checked={draft.notificationsEnabled}
              onChange={(v) => setDraft({ ...draft, notificationsEnabled: v })}
            />
            <SettingToggle
              label="Lancer Giza au démarrage de Windows"
              hint={
                data.capabilities.launchAtStartupAvailable
                  ? "Démarrer l'application avec la session Windows."
                  : "Option enregistrée pour plus tard — non disponible dans cette version."
              }
              checked={draft.launchAtStartup}
              disabled={!data.capabilities.launchAtStartupAvailable}
              onChange={(v) => setDraft({ ...draft, launchAtStartup: v })}
            />
          </ul>
        </section>

        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
          <h2 className="text-sm font-medium text-[var(--color-text)]">Clic droit</h2>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Sur les <strong>doublons</strong>, les <strong>gros fichiers</strong>, les{" "}
            <strong>temporaires</strong> et les <strong>applications</strong> : menu contextuel
            (corbeille, explorateur, copier le chemin, fusion, désinstallation Windows). Les
            suppressions passent par la corbeille ou l&apos;assistant Windows — jamais de
            suppression directe.
          </p>
        </section>

        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
          <h2 className="text-sm font-medium text-[var(--color-text)]">Fusion des doublons</h2>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Règle utilisée par « Tout cocher pour fusion » et « Fusionner tout ». Vous pouvez toujours
            ajuster fichier par fichier avant validation.
          </p>
          <ul className="mt-4 space-y-3">
            {DUPLICATE_KEEP_OPTIONS.map((opt) => (
              <li key={opt.id}>
                <label className="flex cursor-pointer items-start gap-3 text-sm">
                  <input
                    type="radio"
                    name="duplicateKeepRule"
                    className="mt-1"
                    checked={draft.duplicateKeepRule === opt.id}
                    onChange={() => setDraft({ ...draft, duplicateKeepRule: opt.id })}
                  />
                  <span>
                    <span className="text-[var(--color-text)]">{opt.label}</span>
                    <span className="mt-0.5 block text-xs text-[var(--color-text-muted)]">
                      {opt.hint}
                    </span>
                  </span>
                </label>
              </li>
            ))}
          </ul>
        </section>

        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
          <h2 className="text-sm font-medium text-[var(--color-text)]">Seuil des gros fichiers</h2>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Utilisé par l&apos;onglet <strong>Gros fichiers</strong> (valeur par défaut) et le compteur
            du dashboard.
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            {LARGE_FILE_PRESETS_MB.map((preset) => (
              <button
                key={preset.label}
                type="button"
                onClick={() =>
                  setDraft({ ...draft, largeFileThresholdMb: preset.mb })
                }
                className={[
                  "rounded-lg px-3 py-1.5 text-sm transition-colors",
                  draft.largeFileThresholdMb === preset.mb
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
              Personnalisé (Mo)
              <input
                type="number"
                min={1}
                value={customMb}
                onChange={(e) => setCustomMb(e.target.value)}
                className="w-36 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-text)]"
                placeholder="ex. 250"
              />
            </label>
            <button
              type="button"
              onClick={applyCustomThreshold}
              className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text)] hover:border-[var(--color-accent)]"
            >
              Appliquer au brouillon
            </button>
          </div>
          <p className="mt-3 text-xs text-[var(--color-text-muted)]">
            Seuil actuel (brouillon) : <strong>{draft.largeFileThresholdMb.toLocaleString("fr-FR")} Mo</strong>
          </p>
        </section>

        <ExclusionsSection />

        <UpdaterSection />

        <div className="flex flex-wrap gap-3">
          <button
            type="button"
            onClick={() => void handleSave()}
            disabled={saving}
            className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
          >
            {saving ? "Enregistrement…" : "Enregistrer"}
          </button>
          <button
            type="button"
            onClick={() => void load()}
            disabled={saving}
            className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          >
            Annuler les modifications
          </button>
        </div>
      </div>
    </>
  );
}

function SettingToggle({
  label,
  hint,
  checked,
  disabled,
  onChange,
}: {
  label: string;
  hint: string;
  checked: boolean;
  disabled?: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <li className="flex items-start justify-between gap-4">
      <div>
        <p className="text-[var(--color-text)]">{label}</p>
        <p className="mt-1 text-xs text-[var(--color-text-muted)]">{hint}</p>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={[
          "relative h-6 w-11 shrink-0 rounded-full transition-colors disabled:cursor-not-allowed disabled:opacity-40",
          checked ? "bg-[var(--color-accent)]" : "bg-[var(--color-border)]",
        ].join(" ")}
      >
        <span
          className={[
            "absolute top-0.5 h-5 w-5 rounded-full bg-white transition-transform",
            checked ? "left-5" : "left-0.5",
          ].join(" ")}
        />
      </button>
    </li>
  );
}
