import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppInfo,
  DuplicateGroupDetails,
  DuplicateGroupsResult,
  DuplicateMergePlan,
  ApplyOrganizeResult,
  CleanupAnalysisResult,
  ExclusionsOverview,
  DuplicateSnapshot,
  UserExclusion,
  LargeFilesResult,
  OrganizeProposalsResult,
  RecordSnapshotsResult,
  StorageHistoryResult,
  RecommendationsResult,
  NotificationStateOverview,
  ProcessNotificationsResult,
  OperationRecord,
  TrashResult,
  ScanSnapshot,
  SettingsResponse,
  StorageInsights,
  UpdateSettings,
  VolumeStats,
  ScanRootSuggestion,
  BootstrapStatus,
  AnalysisScheduleStatus,
  InstalledAppsResult,
} from "../types/dto";
import { notifyOperationsChanged } from "./operationsRefresh";

export async function fetchAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function fetchVolumeStats(mountPath?: string): Promise<VolumeStats> {
  return invoke<VolumeStats>("get_volume_stats", { mountPath: mountPath ?? null });
}

export async function fetchMountedVolumes(): Promise<VolumeStats[]> {
  return invoke<VolumeStats[]>("list_mounted_volumes");
}

export async function fetchScanRootSuggestions(): Promise<ScanRootSuggestion[]> {
  return invoke<ScanRootSuggestion[]>("get_scan_root_suggestions");
}

export async function fetchInstalledApps(limit = 24): Promise<InstalledAppsResult> {
  return invoke<InstalledAppsResult>("list_installed_apps", { limit });
}

export async function ensureBootstrapAnalysis(): Promise<BootstrapStatus> {
  return invoke<BootstrapStatus>("ensure_bootstrap_analysis");
}

export async function fetchBootstrapStatus(): Promise<BootstrapStatus> {
  return invoke<BootstrapStatus>("get_bootstrap_status");
}

export async function fetchAnalysisScheduleStatus(): Promise<AnalysisScheduleStatus> {
  return invoke<AnalysisScheduleStatus>("get_analysis_schedule_status");
}

export async function requestVolumeRescan(forceAll = false): Promise<BootstrapStatus> {
  const status = await invoke<BootstrapStatus>("request_volume_rescan", { forceAll });
  window.dispatchEvent(new CustomEvent(ANALYSIS_REFRESH_EVENT));
  return status;
}

export const ANALYSIS_REFRESH_EVENT = "giza-analysis-refresh";

export async function fetchStorageInsights(): Promise<StorageInsights> {
  return invoke<StorageInsights>("get_storage_insights");
}

export async function fetchRecommendations(options?: {
  includeCleanupScan?: boolean;
}): Promise<RecommendationsResult> {
  return invoke<RecommendationsResult>("get_recommendations", {
    includeCleanupScan: options?.includeCleanupScan ?? null,
  });
}

export async function fetchNotificationState(): Promise<NotificationStateOverview> {
  return invoke<NotificationStateOverview>("get_notification_state");
}

export async function processNotifications(): Promise<ProcessNotificationsResult> {
  return invoke<ProcessNotificationsResult>("process_notifications");
}

export async function fetchSettings(): Promise<SettingsResponse> {
  return invoke<SettingsResponse>("get_settings");
}

export async function saveSettings(patch: UpdateSettings): Promise<SettingsResponse> {
  return invoke<SettingsResponse>("update_settings", { patch });
}

export async function fetchExclusions(): Promise<ExclusionsOverview> {
  return invoke<ExclusionsOverview>("get_exclusions");
}

export async function addUserExclusion(path: string): Promise<UserExclusion> {
  return invoke<UserExclusion>("add_user_exclusion", { path });
}

export async function removeUserExclusion(id: number): Promise<boolean> {
  return invoke<boolean>("remove_user_exclusion", { id });
}

export async function resetUserExclusions(): Promise<number> {
  return invoke<number>("reset_user_exclusions");
}

export async function recordStorageSnapshots(force?: boolean): Promise<RecordSnapshotsResult> {
  return invoke<RecordSnapshotsResult>("record_storage_snapshots", { force: force ?? null });
}

export async function fetchStorageHistory(options?: {
  mountPath?: string;
  periodDays?: number;
  limit?: number;
}): Promise<StorageHistoryResult> {
  return invoke<StorageHistoryResult>("get_storage_history", {
    mountPath: options?.mountPath ?? null,
    periodDays: options?.periodDays ?? null,
    limit: options?.limit ?? null,
  });
}

export async function startScan(rootPath: string): Promise<ScanSnapshot> {
  return invoke<ScanSnapshot>("start_scan", { rootPath });
}

export async function cancelScan(): Promise<ScanSnapshot> {
  return invoke<ScanSnapshot>("cancel_scan");
}

export async function getScanStatus(): Promise<ScanSnapshot> {
  return invoke<ScanSnapshot>("get_scan_status");
}

export type ScanProgressPayload = {
  scanId: number;
  filesSeen: number;
  bytesTotal: number;
  currentPath?: string;
  status: string;
};

export type ScanFinishedPayload = {
  scanId: number;
  status: string;
  filesSeen: number;
  bytesTotal: number;
  errorMessage?: string;
};

export function onScanProgress(handler: (payload: ScanProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ScanProgressPayload>("scan-progress", (event) => handler(event.payload));
}

export function onScanFinished(handler: (payload: ScanFinishedPayload) => void): Promise<UnlistenFn> {
  return listen<ScanFinishedPayload>("scan-finished", (event) => handler(event.payload));
}

export async function startDuplicateAnalysis(scanId?: number): Promise<DuplicateSnapshot> {
  return invoke<DuplicateSnapshot>("start_duplicate_analysis", { scanId: scanId ?? null });
}

export async function cancelDuplicateAnalysis(): Promise<DuplicateSnapshot> {
  return invoke<DuplicateSnapshot>("cancel_duplicate_analysis");
}

export async function getDuplicateStatus(): Promise<DuplicateSnapshot> {
  return invoke<DuplicateSnapshot>("get_duplicate_status");
}

export async function fetchDuplicateGroups(scanId?: number): Promise<DuplicateGroupsResult> {
  return invoke<DuplicateGroupsResult>("list_duplicate_groups", {
    scanId: scanId ?? null,
    limit: null,
  });
}

export async function fetchDuplicateGroupDetails(
  groupId: number,
): Promise<DuplicateGroupDetails | null> {
  return invoke<DuplicateGroupDetails | null>("get_duplicate_group_details", { groupId });
}

export async function fetchDuplicateMergePlan(
  scanId?: number,
): Promise<DuplicateMergePlan | null> {
  return invoke<DuplicateMergePlan | null>("get_duplicate_merge_plan", {
    scanId: scanId ?? null,
  });
}

export async function showItemInFolder(path: string): Promise<void> {
  return invoke<void>("show_item_in_folder", { path });
}

export async function uninstallApplication(
  appName: string,
  options?: { publisher?: string; estimatedSizeBytes?: number },
): Promise<void> {
  await invoke<void>("uninstall_application", {
    appName,
    publisher: options?.publisher ?? null,
    estimatedSizeBytes: options?.estimatedSizeBytes ?? null,
  });
  notifyOperationsChanged();
}

export function onDuplicateProgress(
  handler: (payload: DuplicateProgressPayload) => void,
): Promise<UnlistenFn> {
  return listen<DuplicateProgressPayload>("duplicate-progress", (event) => handler(event.payload));
}

export function onDuplicateFinished(
  handler: (payload: DuplicateFinishedPayload) => void,
): Promise<UnlistenFn> {
  return listen<DuplicateFinishedPayload>("duplicate-finished", (event) =>
    handler(event.payload),
  );
}

export type DuplicateProgressPayload = {
  scanId: number;
  phase: string;
  candidatesHashed: number;
  groupsFound: number;
  recoverableBytes: number;
  status: string;
};

export type DuplicateFinishedPayload = {
  scanId: number;
  status: string;
  groupsFound: number;
  recoverableBytes: number;
  errorMessage?: string;
};

export async function moveFilesToTrash(paths: string[]): Promise<TrashResult> {
  const result = await invoke<TrashResult>("move_files_to_trash", { paths });
  if (result.successCount > 0) notifyOperationsChanged();
  return result;
}

export async function fetchOperations(limit?: number): Promise<OperationRecord[]> {
  return invoke<OperationRecord[]>("list_operations", { limit: limit ?? null });
}

export async function analyzeTempFiles(maxFiles?: number): Promise<CleanupAnalysisResult> {
  return invoke<CleanupAnalysisResult>("analyze_temp_files", {
    maxFiles: maxFiles ?? null,
  });
}

export async function fetchOrganizeProposals(scanId?: number): Promise<OrganizeProposalsResult> {
  return invoke<OrganizeProposalsResult>("get_organize_proposals", {
    scanId: scanId ?? null,
  });
}

export async function applyOrganizeMoves(
  moves: { sourcePath: string; destPath: string }[],
): Promise<ApplyOrganizeResult> {
  const result = await invoke<ApplyOrganizeResult>("apply_organize_moves", { moves });
  if (result.successCount > 0) notifyOperationsChanged();
  return result;
}

export async function fetchLargeFiles(
  minSizeBytes: number,
  scanId?: number,
): Promise<LargeFilesResult> {
  return invoke<LargeFilesResult>("list_large_files", {
    minSizeBytes,
    scanId: scanId ?? null,
    limit: null,
  });
}
