import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

function base(props: IconProps) {
  return {
    width: 18,
    height: 18,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.75,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
    ...props,
  };
}

export function IconDashboard(p: IconProps) {
  return (
    <svg {...base(p)}>
      <rect x="3" y="3" width="7" height="9" rx="1.5" />
      <rect x="14" y="3" width="7" height="5" rx="1.5" />
      <rect x="14" y="12" width="7" height="9" rx="1.5" />
      <rect x="3" y="16" width="7" height="5" rx="1.5" />
    </svg>
  );
}

export function IconAnalyze(p: IconProps) {
  return (
    <svg {...base(p)}>
      <circle cx="11" cy="11" r="7" />
      <path d="M20 20l3 3" />
    </svg>
  );
}

export function IconDuplicates(p: IconProps) {
  return (
    <svg {...base(p)}>
      <rect x="8" y="4" width="12" height="14" rx="2" />
      <rect x="4" y="8" width="12" height="14" rx="2" />
    </svg>
  );
}

export function IconLargeFiles(p: IconProps) {
  return (
    <svg {...base(p)}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <path d="M14 2v6h6M12 18v-6M9 15h6" />
    </svg>
  );
}

export function IconCleanup(p: IconProps) {
  return (
    <svg {...base(p)}>
      <path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14" />
    </svg>
  );
}

export function IconOrganize(p: IconProps) {
  return (
    <svg {...base(p)}>
      <path d="M3 7h7v7H3zM14 7h7v4h-7zM14 14h7v7h-7zM3 16h7v5H3z" />
    </svg>
  );
}

export function IconHistory(p: IconProps) {
  return (
    <svg {...base(p)}>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </svg>
  );
}

export function IconSettings(p: IconProps) {
  return (
    <svg {...base(p)}>
      <circle cx="12" cy="12" r="3" />
      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
    </svg>
  );
}

export const NAV_ICONS = {
  "/dashboard": IconDashboard,
  "/analyze": IconAnalyze,
  "/duplicates": IconDuplicates,
  "/large-files": IconLargeFiles,
  "/cleanup": IconCleanup,
  "/organize": IconOrganize,
  "/history": IconHistory,
  "/settings": IconSettings,
} as const;

export function RecommendationTypeIcon({ type }: { type: string }) {
  switch (type) {
    case "duplicates":
      return <IconDuplicates className="text-violet-400" />;
    case "large_files":
      return <IconLargeFiles className="text-sky-400" />;
    case "cleanup":
      return <IconCleanup className="text-emerald-400" />;
    case "applications":
      return <IconApps className="text-orange-400" />;
    case "storage_usage":
    case "storage_growth":
      return <IconDisk className="text-amber-400" />;
    default:
      return <IconSpark className="text-[var(--color-text-muted)]" />;
  }
}

function IconApps(p: IconProps) {
  return (
    <svg {...base(p)}>
      <rect x="3" y="3" width="8" height="8" rx="2" />
      <rect x="13" y="3" width="8" height="8" rx="2" />
      <rect x="3" y="13" width="8" height="8" rx="2" />
      <rect x="13" y="13" width="8" height="8" rx="2" />
    </svg>
  );
}

function IconDisk(p: IconProps) {
  return (
    <svg {...base(p)}>
      <ellipse cx="12" cy="6" rx="9" ry="3" />
      <path d="M3 6v12c0 1.66 4.03 3 9 3s9-1.34 9-3V6" />
      <path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3" />
    </svg>
  );
}

function IconSpark(p: IconProps) {
  return (
    <svg {...base(p)}>
      <path d="M12 2l1.5 5.5L19 9l-5.5 1.5L12 16l-1.5-5.5L5 9l5.5-1.5z" />
    </svg>
  );
}

export function QuickActionIcon({ route }: { route: string }) {
  const Icon = NAV_ICONS[route as keyof typeof NAV_ICONS] ?? IconSpark;
  return (
    <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-[var(--color-accent)]/15 text-[var(--color-accent)]">
      <Icon />
    </span>
  );
}
