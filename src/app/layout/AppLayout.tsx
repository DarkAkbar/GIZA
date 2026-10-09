import { Outlet } from "react-router-dom";
import { BootstrapStatusBanner } from "../../components/BootstrapStatusBanner";
import { useNotificationDispatch } from "../../hooks/useNotificationDispatch";
import { Sidebar } from "./Sidebar";

export function AppLayout() {
  useNotificationDispatch();

  return (
    <div className="flex h-full min-h-screen bg-[var(--color-surface)]">
      <Sidebar />
      <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <header className="dp-titlebar flex items-center justify-between">
          <p className="text-xs text-[var(--color-text-muted)]">
            Gestion intelligente du stockage — Windows
          </p>
        </header>
        <div className="flex-1 overflow-y-auto p-6 md:p-8">
          <BootstrapStatusBanner />
          <Outlet />
        </div>
      </main>
    </div>
  );
}
