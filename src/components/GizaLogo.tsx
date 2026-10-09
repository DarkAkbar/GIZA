import gizaLogo from "../assets/giza-logo.png";

export type GizaLogoMotion = false | "idle" | "active";

type GizaLogoProps = {
  size?: number;
  motion?: GizaLogoMotion;
  className?: string;
  alt?: string;
};

export function GizaLogo({
  size = 44,
  motion = "idle",
  className = "",
  alt = "Giza",
}: GizaLogoProps) {
  const motionClass =
    motion === "active"
      ? "giza-logo--active"
      : motion === "idle"
        ? "giza-logo--idle"
        : "";

  return (
    <div
      className={`giza-logo ${motionClass} ${className}`.trim()}
      style={{ width: size, height: size }}
      aria-hidden={alt === ""}
    >
      <span className="giza-logo__glow" />
      <span className="giza-logo__ring" />
      <img src={gizaLogo} alt={alt} className="giza-logo__img" width={size} height={size} />
    </div>
  );
}
