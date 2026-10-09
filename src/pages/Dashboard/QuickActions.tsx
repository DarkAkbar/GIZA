import { Link } from "react-router-dom";
import { QuickActionIcon } from "../../components/icons/NavIcons";

const ACTIONS = [
  { to: "/analyze", label: "Analyser le stockage", description: "Indexer un dossier" },
  { to: "/large-files", label: "Gros fichiers", description: "Dernier scan" },
  { to: "/duplicates", label: "Doublons", description: "Examiner les copies" },
  { to: "/cleanup", label: "Nettoyer", description: "Temporaires et caches" },
  { to: "/organize", label: "Organiser", description: "Propositions de classement" },
  { to: "/history", label: "Historique", description: "Opérations passées" },
] as const;

export function QuickActions() {
  return (
    <section className="dp-card p-5">
      <h2 className="text-sm font-medium text-[var(--color-text)]">Actions rapides</h2>
      <p className="mt-1 text-xs text-[var(--color-text-muted)]">
        Accès direct aux fonctionnalités — aucune action destructive depuis ici.
      </p>
      <ul className="mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
        {ACTIONS.map((action) => (
          <li key={action.to}>
            <Link
              to={action.to}
              className="flex h-full items-start gap-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]/80 px-4 py-3 transition hover:border-[var(--color-accent)]/45 hover:bg-[var(--color-surface-elevated)]"
            >
              <QuickActionIcon route={action.to} />
              <span className="min-w-0">
                <span className="block text-sm font-medium text-[var(--color-text)]">
                  {action.label}
                </span>
                <span className="mt-1 block text-xs text-[var(--color-text-muted)]">
                  {action.description}
                </span>
              </span>
            </Link>
          </li>
        ))}
      </ul>
    </section>
  );
}
