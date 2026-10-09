import { useCallback, useEffect, useMemo, useState } from "react";
import { PageHeader } from "../../components/PageHeader";

import {
  ANALYSIS_REFRESH_EVENT,
  fetchAppInfo,
  fetchRecommendations,
  fetchSettings,
  fetchStorageInsights,
  fetchVolumeStats,
  recordStorageSnapshots,
} from "../../lib/tauri";

import { formatUserError } from "../../lib/userError";
import type { AppInfo, RecommendationDto, StorageInsights, VolumeStats } from "../../types/dto";
import { deriveStorageStatus } from "./dashboardSummary";
import { InstalledAppsPanel } from "../../components/InstalledAppsPanel";
import { QuickActions } from "./QuickActions";
import { RecommendationsSection } from "./RecommendationsSection";
import { StorageEvolutionSection } from "./StorageEvolutionSection";
import { StorageOverviewCard } from "./StorageOverviewCard";
import { StorageStatsSection } from "./StorageStatsSection";
import { AnalysisReminderBanner } from "../../components/AnalysisReminderBanner";
import { StorageStatusBanner } from "./StorageStatusBanner";

export function DashboardPage() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [infoError, setInfoError] = useState<string | null>(null);

  const [volume, setVolume] = useState<VolumeStats | null>(null);
  const [volumeLoading, setVolumeLoading] = useState(true);
  const [volumeError, setVolumeError] = useState<string | null>(null);

  const [insights, setInsights] = useState<StorageInsights | null>(null);
  const [insightsLoading, setInsightsLoading] = useState(true);
  const [insightsError, setInsightsError] = useState<string | null>(null);

  const [recommendations, setRecommendations] = useState<RecommendationDto[]>([]);
  const [recommendationsLoading, setRecommendationsLoading] = useState(true);
  const [recommendationsError, setRecommendationsError] = useState<string | null>(null);

  const [largeFileThresholdMb, setLargeFileThresholdMb] = useState(100);
  const [refreshSignal, setRefreshSignal] = useState(0);
  const [refreshing, setRefreshing] = useState(false);

  const loadVolume = useCallback(async () => {
    setVolumeLoading(true);
    setVolumeError(null);
    try {
      setVolume(await fetchVolumeStats());
    } catch (err) {
      setVolume(null);
      setVolumeError(formatUserError(err));
    } finally {
      setVolumeLoading(false);
    }
  }, []);

  const loadInsights = useCallback(async () => {
    setInsightsLoading(true);
    setInsightsError(null);
    try {
      setInsights(await fetchStorageInsights());
    } catch (err) {
      setInsights(null);
      setInsightsError(formatUserError(err));
    } finally {
      setInsightsLoading(false);
    }
  }, []);

  const loadRecommendations = useCallback(async () => {
    setRecommendationsLoading(true);
    setRecommendationsError(null);
    try {
      const result = await fetchRecommendations();
      setRecommendations(result.recommendations);
    } catch (err) {
      setRecommendations([]);
      setRecommendationsError(formatUserError(err));
    } finally {
      setRecommendationsLoading(false);
    }
  }, []);

  const loadAppInfo = useCallback(async () => {
    setInfoError(null);
    try {
      setInfo(await fetchAppInfo());
    } catch {
      setInfo(null);
      setInfoError("Informations application indisponibles.");
    }
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      const settingsResponse = await fetchSettings();
      setLargeFileThresholdMb(settingsResponse.settings.largeFileThresholdMb);
    } catch {
      /* seuil par défaut */
    }
  }, []);

  const refreshDashboard = useCallback(async () => {
    setRefreshing(true);
    try {
      await recordStorageSnapshots(false);
    } catch {
      /* cooldown ou erreur non bloquante */
    }
    await Promise.all([
      loadVolume(),
      loadInsights(),
      loadRecommendations(),
      loadAppInfo(),
      loadSettings(),
    ]);
    setRefreshSignal((n) => n + 1);
    setRefreshing(false);
  }, [loadAppInfo, loadInsights, loadRecommendations, loadSettings, loadVolume]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        await recordStorageSnapshots(false);
      } catch {
        /* non bloquant */
      }
      if (cancelled) return;
      await loadVolume();
      await loadSettings();
      if (cancelled) return;
      await Promise.all([loadInsights(), loadRecommendations()]);
      if (cancelled) return;
      void loadAppInfo();
    })();
    return () => {
      cancelled = true;
    };
  }, [loadAppInfo, loadInsights, loadRecommendations, loadSettings, loadVolume]);

  useEffect(() => {
    let refreshTimer: number | null = null;
    const onRefresh = () => {
      if (refreshTimer !== null) return;
      refreshTimer = window.setTimeout(() => {
        refreshTimer = null;
        void Promise.all([loadInsights(), loadRecommendations()]);
      }, 800);
    };
    window.addEventListener(ANALYSIS_REFRESH_EVENT, onRefresh);
    return () => {
      window.removeEventListener(ANALYSIS_REFRESH_EVENT, onRefresh);
      if (refreshTimer !== null) window.clearTimeout(refreshTimer);
    };
  }, [loadInsights, loadRecommendations]);

  const volumeLabel = volume
    ? `Volume ${volume.mountPath.replace(/\\$/, "")}`
    : "Volume système";

  const statusSummary = useMemo(
    () =>
      deriveStorageStatus(
        volume,
        recommendationsLoading ? null : recommendations,
        recommendationsError !== null,
      ),
    [volume, recommendations, recommendationsLoading, recommendationsError],
  );

  return (
    <>
      <div className="mb-6 flex flex-wrap items-start justify-between gap-3">
        <PageHeader
          title="Dashboard"
          description="Voici l'état actuel de votre stockage."
        />
        <button
          type="button"
          onClick={() => void refreshDashboard()}
          disabled={refreshing}
          className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-elevated)] px-4 py-2 text-sm text-[var(--color-text)] hover:border-[var(--color-accent)]/40 disabled:opacity-50"
        >
          {refreshing ? "Actualisation…" : "Actualiser"}
        </button>
      </div>

      <div className="flex flex-col gap-4">
        <AnalysisReminderBanner />

        <StorageStatusBanner
          summary={statusSummary}
          loading={volumeLoading && recommendationsLoading}
        />

        <StorageOverviewCard
          volume={volume}
          loading={volumeLoading}
          error={volumeError}
          statusLevel={statusSummary.level}
          volumeLabel={volumeLabel}
        />

        <RecommendationsSection
          items={recommendations}
          loading={recommendationsLoading}
          error={recommendationsError}
          onRetry={() => void loadRecommendations()}
        />

        <StorageEvolutionSection
          defaultMountPath={volume?.mountPath}
          recordSnapshotOnLoad={false}
          refreshSignal={refreshSignal}
        />

        <StorageStatsSection
          insights={insights}
          insightsLoading={insightsLoading}
          insightsError={insightsError}
          largeFileThresholdMb={largeFileThresholdMb}
        />

        <QuickActions />

        <InstalledAppsPanel limit={8} compact />
      </div>

      <section className="mt-8 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-5">
        <h3 className="text-sm font-medium text-[var(--color-text)]">À propos</h3>
        {info ? (
          <dl className="mt-3 grid gap-2 text-sm text-[var(--color-text-muted)] sm:grid-cols-2">
            <div>
              <dt className="text-[var(--color-text-muted)]">Version</dt>
              <dd className="text-[var(--color-text)]">
                {info.name} {info.version}
              </dd>
            </div>
            <div>
              <dt className="text-[var(--color-text-muted)]">Plateforme</dt>
              <dd className="text-[var(--color-text)]">{info.platform}</dd>
            </div>
            <div className="sm:col-span-2">
              <dt className="text-[var(--color-text-muted)]">Base locale</dt>
              <dd className="break-all font-mono text-xs text-[var(--color-text)]">{info.dbPath}</dd>
            </div>
          </dl>
        ) : (
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            {infoError ?? "Chargement…"}
          </p>
        )}
      </section>
    </>
  );
}
