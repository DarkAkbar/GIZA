import { NavLink } from "react-router-dom";
import { GizaMark } from "../../components/GizaMark";
import { NAV_ICONS } from "../../components/icons/NavIcons";

const navItems = [
  { to: "/dashboard", label: "Dashboard" },
  { to: "/analyze", label: "Analyse" },
  { to: "/duplicates", label: "Doublons" },
  { to: "/large-files", label: "Gros fichiers" },
  { to: "/cleanup", label: "Nettoyage" },
  { to: "/organize", label: "Organisation" },
  { to: "/history", label: "Historique" },
  { to: "/settings", label: "Paramètres" },
] as const;

export function Sidebar() {
  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-[var(--color-border)] bg-[var(--color-surface-elevated)]/95 py-5 backdrop-blur-md">
      <div className="mb-8 px-4">
        <div className="flex items-center gap-3">
          <GizaMark size={44} spin={false} />
          <div className="min-w-0">
            <p className="text-[10px] font-semibold uppercase tracking-widest text-orange-400/90">
              Assistant stockage
            </p>
            <h1 className="bg-gradient-to-r from-white to-orange-200/90 bg-clip-text text-xl font-semibold text-transparent">
              Giza
            </h1>
          </div>
        </div>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5 px-2">
        {navItems.map((item) => {
          const Icon = NAV_ICONS[item.to];
          return (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                ["dp-nav-link", isActive ? "dp-nav-link-active" : "dp-nav-link-idle"].join(" ")
              }
            >
              <Icon className="shrink-0 opacity-90" />
              <span>{item.label}</span>
            </NavLink>
          );
        })}
      </nav>
      <p className="mt-4 px-4 text-[11px] leading-snug text-[var(--color-text-muted)]">
        Analyser d&apos;abord. Agir seulement avec votre accord.
      </p>
    </aside>
  );
}
