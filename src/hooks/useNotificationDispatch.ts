import { useEffect, useRef } from "react";

import { listen } from "@tauri-apps/api/event";

import { processNotifications } from "../lib/tauri";

const REMINDER_CHECK_MS = 12 * 60 * 60 * 1000;

/** Notifications locales au démarrage, après analyse et périodiquement (rappel ré-analyse). */
export function useNotificationDispatch() {
  const startupDone = useRef(false);

  useEffect(() => {
    const run = () => {
      void processNotifications().catch(() => {
        /* permission refusée ou hors Tauri */
      });
    };

    const timer = window.setTimeout(() => {
      if (startupDone.current) return;
      startupDone.current = true;
      run();
    }, 3000);

    const periodic = window.setInterval(run, REMINDER_CHECK_MS);

    let unlisten: (() => void) | undefined;

    void listen<{ scanId: number; status: string }>("scan-finished", (event) => {
      if (event.payload.status === "completed") {
        run();
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      window.clearTimeout(timer);
      window.clearInterval(periodic);
      unlisten?.();
    };
  }, []);
}
