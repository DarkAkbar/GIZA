import type { ReactNode } from "react";

export type InlineAlertVariant = "error" | "warning" | "success" | "info";

const STYLES: Record<InlineAlertVariant, string> = {
  error: "border-red-500/40 bg-red-500/10 text-red-200",
  warning: "border-amber-500/40 bg-amber-500/10 text-amber-100",
  success: "border-emerald-500/40 bg-emerald-500/10 text-emerald-100",
  info: "border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-muted)]",
};

type InlineAlertProps = {
  variant: InlineAlertVariant;
  title?: string;
  children: ReactNode;
  onRetry?: () => void;
  retryLabel?: string;
  className?: string;
};

export function InlineAlert({
  variant,
  title,
  children,
  onRetry,
  retryLabel = "Réessayer",
  className = "",
}: InlineAlertProps) {
  return (
    <div
      role={variant === "error" ? "alert" : "status"}
      className={`rounded-lg border px-4 py-3 text-sm ${STYLES[variant]} ${className}`}
    >
      {title ? <p className="font-medium text-[var(--color-text)]">{title}</p> : null}
      <div className={title ? "mt-1" : undefined}>{children}</div>
      {onRetry ? (
        <button
          type="button"
          onClick={onRetry}
          className="mt-2 text-sm font-medium text-[var(--color-accent)] hover:underline"
        >
          {retryLabel}
        </button>
      ) : null}
    </div>
  );
}
