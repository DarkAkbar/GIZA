type AppIconProps = {
  name: string;
  iconDataUrl?: string | null;
  size?: "sm" | "md" | "lg";
  className?: string;
};

const sizes = {
  sm: "h-8 w-8 text-xs",
  md: "h-10 w-10 text-sm",
  lg: "h-12 w-12 text-base",
};

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[1][0]).toUpperCase();
}

function hueFromName(name: string): number {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) % 360;
  return h;
}

export function AppIcon({ name, iconDataUrl, size = "md", className = "" }: AppIconProps) {
  const box = sizes[size];
  if (iconDataUrl) {
    return (
      <img
        src={iconDataUrl}
        alt=""
        className={`${box} shrink-0 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] object-contain p-0.5 ${className}`}
      />
    );
  }

  const hue = hueFromName(name);
  return (
    <span
      className={`${box} inline-flex shrink-0 items-center justify-center rounded-xl border border-white/10 font-semibold text-white ${className}`}
      style={{
        background: `linear-gradient(135deg, hsl(${hue} 70% 45%), hsl(${(hue + 40) % 360} 65% 35%))`,
      }}
      aria-hidden
    >
      {initials(name)}
    </span>
  );
}
