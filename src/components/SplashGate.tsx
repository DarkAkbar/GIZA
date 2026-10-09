import { useEffect, useState, type ReactNode } from "react";
import { GizaMark } from "./GizaMark";
import { ANALYSIS_REFRESH_EVENT, ensureBootstrapAnalysis } from "../lib/tauri";

const MIN_SPLASH_MS = 1500;
const BOOTSTRAP_STARTED_KEY = "giza.bootstrap.started";

type SplashGateProps = {
  children: ReactNode;
};

function dispatchRefresh() {
  window.dispatchEvent(new CustomEvent(ANALYSIS_REFRESH_EVENT));
}

function SplashScreen({ exiting }: { exiting: boolean }) {
  const reduceMotion =
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  return (
    <div
      className={[
        "fixed inset-0 z-[100] flex flex-col items-center justify-center bg-[var(--color-surface)] transition-opacity duration-500",
        exiting ? "pointer-events-none opacity-0" : "opacity-100",
      ].join(" ")}
      aria-busy={!exiting}
      aria-label="Chargement de Giza"
    >
      <div
        className="absolute inset-0 opacity-60"
        style={{
          background:
            "radial-gradient(ellipse 80% 60% at 50% 40%, rgba(249, 115, 22, 0.18) 0%, transparent 55%)",
        }}
      />
      <GizaMark size={128} spin={!reduceMotion} />
      <p className="relative mt-8 text-sm font-medium text-[var(--color-text)]">Giza</p>
      <p className="relative mt-1 text-xs text-[var(--color-text-muted)]">
        Préparation de l&apos;analyse en arrière-plan…
      </p>
    </div>
  );
}

export function SplashGate({ children }: SplashGateProps) {
  const [phase, setPhase] = useState<"splash" | "exit" | "app">("splash");

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      try {
        sessionStorage.setItem(BOOTSTRAP_STARTED_KEY, "1");
        await ensureBootstrapAnalysis();
        dispatchRefresh();
      } catch {
        /* Tauri indisponible ou analyse déjà en cours */
      }

      const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
      const waitMs = reduceMotion ? 450 : MIN_SPLASH_MS;
      await new Promise((resolve) => window.setTimeout(resolve, waitMs));

      if (cancelled) return;
      setPhase("exit");
      window.setTimeout(() => {
        if (!cancelled) setPhase("app");
      }, 480);
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <>
      {phase !== "app" ? <SplashScreen exiting={phase === "exit"} /> : null}
      <div
        className={[
          "h-full min-h-screen transition-opacity duration-500",
          phase === "app" ? "opacity-100" : "pointer-events-none opacity-0",
        ].join(" ")}
        aria-hidden={phase !== "app"}
        inert={phase !== "app" ? true : undefined}
      >
        {children}
      </div>
    </>
  );
}

export function wasBootstrapStartedInSession(): boolean {
  try {
    return sessionStorage.getItem(BOOTSTRAP_STARTED_KEY) === "1";
  } catch {
    return false;
  }
}
