type GizaMarkProps = {
  size?: number;
  /** Losange orange en rotation ; le point jaune reste fixe. */
  spin?: boolean;
  className?: string;
};

export function GizaMark({ size = 96, spin = false, className = "" }: GizaMarkProps) {
  return (
    <div
      className={`relative inline-flex shrink-0 items-center justify-center ${className}`.trim()}
      style={{ width: size, height: size }}
      role="img"
      aria-label="Giza"
    >
      <div
        className={[
          "absolute inset-0 flex items-center justify-center",
          spin ? "giza-mark-diamond--spin" : "",
        ].join(" ")}
      >
        <svg
          viewBox="0 0 100 100"
          width={size}
          height={size}
          className="block"
          aria-hidden
        >
          <rect
            x="16"
            y="16"
            width="68"
            height="68"
            rx="17"
            fill="#f97316"
            transform="rotate(45 50 50)"
          />
        </svg>
      </div>
      <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
        <svg viewBox="0 0 100 100" width={size} height={size} className="block" aria-hidden>
          <circle cx="50" cy="50" r="21" fill="#0a0e14" />
          <circle cx="50" cy="50" r="21" fill="none" stroke="#0a0e14" strokeWidth="1" />
          <circle cx="50" cy="50" r="9.5" fill="#facc15" />
        </svg>
      </div>
    </div>
  );
}
