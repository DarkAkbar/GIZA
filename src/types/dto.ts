export type ScanSnapshot = {
  scanId?: number;
  rootPath?: string;
  status: string;
  filesSeen: number;
  bytesTotal: number;
  currentPath?: string;
  errorMessage?: string;
};

export type VolumeStats = {
  mountPath: string;
  totalBytes: number;
  availableBytes: number;
  usedBytes: number;
  usagePercent: number;
};

export type ScanRootSuggestion = {
  mountPath: string;
  label: string;
  path: string;
  scope: "user_profile" | "user_folder" | "volume" | string;
};

export type AppInfo = {
  name: string;
  version: string;
  dbPath: string;
  migrationVersion: number;
  platform: string;
};

export type InstalledApp = {
  name: string;
  publisher?: string;
  estimatedSizeBytes?: number;
  iconDataUrl?: string;
  canUninstall?: boolean;
};

export type InstalledAppsResult = {
  apps: InstalledApp[];
  platformSupported: boolean;
};

export type BootstrapStatus = {
  phase: "idle" | "scanning" | "duplicates" | "done" | string;
  volumesRemaining: number;
  currentRoot?: string;
  message: string;
};

export type RiskLevel = "SAFE" | "REVIEW" | "RISKY";

export type ScanSummaryDto = {
  id: number;
  rootPath: string;
  filesSeen: number;
  bytesTotal: number;
  finishedAt?: string;
};

export type DuplicateKeepRule = "shortestPath" | "newest" | "oldest";

export type AppSettings = {
  largeFileThresholdMb: number;
  confirmSensitiveActions: boolean;
  notificationsEnabled: boolean;
  launchAtStartup: boolean;
  duplicateKeepRule: DuplicateKeepRule;
  analysisReminderEnabled: boolean;
  analysisReminderIntervalDays: number;
};

export type AnalysisScheduleStatus = {
  reminderEnabled: boolean;
  intervalDays: number;
  lastScanFinishedAt?: string;
  daysSinceLastScan?: number;
  stale: boolean;
  hasEverScanned: boolean;
};

export type SettingsCapabilities = {
  launchAtStartupAvailable: boolean;
};

export type SettingsResponse = {
  settings: AppSettings;
  capabilities: SettingsCapabilities;
};

export type UserExclusion = {
  id: number;
  path: string;
  createdAt: string;
};

export type SystemExclusionInfo = {
  name: string;
  protected: boolean;
};

export type StorageSnapshotDto = {
  id: number;
  mountPath: string;
  totalBytes: number;
  usedBytes: number;
  freeBytes: number;
  usagePercent: number;
  createdAt: string;
};

export type RecordSnapshotsResult = {
  recordedCount: number;
  skippedCooldownCount: number;
  volumesSeen: number;
  snapshots: StorageSnapshotDto[];
};

export type StorageHistoryResult = {
  mountPath: string;
  availableMounts: string[];
  periodDays: number;
  snapshots: StorageSnapshotDto[];
};

export type ExclusionsOverview = {
  system: SystemExclusionInfo[];
  user: UserExclusion[];
};

export type UpdateSettings = {
  largeFileThresholdMb?: number;
  confirmSensitiveActions?: boolean;
  notificationsEnabled?: boolean;
  launchAtStartup?: boolean;
  duplicateKeepRule?: DuplicateKeepRule;
  analysisReminderEnabled?: boolean;
  analysisReminderIntervalDays?: number;
};

export type StorageInsights = {
  scan?: ScanSummaryDto;
  hasDuplicateAnalysis: boolean;
  duplicateGroupCount: number;
  duplicateRecoverableBytes: number;
  largeFilesCount: number;
  largeFilesTotalBytes: number;
};

export type RecommendationDto = {
  id: string;
  recommendationType: string;
  severity: "info" | "warning" | "important" | string;
  title: string;
  description: string;
  reason: string;
  potentialBytes: number;
  actionLabel: string;
  actionRoute: string;
  detectedValue?: number;
  threshold?: number;
  periodDays?: number;
  mountPath?: string;
  createdAt: string;
};

export type RecommendationsResult = {
  recommendations: RecommendationDto[];
  generatedAt: string;
};

export type NotificationStateOverview = {
  enabled: boolean;
  dedupeEntries: number;
};

export type ProcessNotificationsResult = {
  enabled: boolean;
  sentCount: number;
  sentIds: string[];
  skippedDuplicate: number;
};

export type LargeFileItem = {
  id: number;
  path: string;
  name: string;
  extension?: string;
  sizeBytes: number;
  modifiedAt?: string;
  mimeOrKind?: string;
};

export type DuplicateGroup = {
  id: number;
  scanId: number;
  contentHash: string;
  fileCount: number;
  totalBytes: number;
  recoverableBytes: number;
};

export type DuplicateMember = {
  fileId: number;
  path: string;
  name: string;
  sizeBytes: number;
  modifiedAt?: string;
};

export type DuplicateGroupsResult = {
  scan?: ScanSummaryDto;
  groupCount: number;
  recoverableBytes: number;
  groups: DuplicateGroup[];
};

export type DuplicateGroupDetails = {
  group: DuplicateGroup;
  members: DuplicateMember[];
};

export type DuplicateMergeItem = {
  groupId: number;
  path: string;
  sizeBytes: number;
  keepPath: string;
};

export type DuplicateMergePlan = {
  scanId: number;
  toRemove: DuplicateMergeItem[];
  totalBytes: number;
};

export type DuplicateSnapshot = {
  scanId?: number;
  status: string;
  candidatesHashed: number;
  groupsFound: number;
  recoverableBytes: number;
  errorMessage?: string;
};

export type LargeFilesResult = {
  scan?: ScanSummaryDto;
  minSizeBytes: number;
  matchCount: number;
  matchTotalBytes: number;
  items: LargeFileItem[];
};

export type TrashItemOutcome = {
  path: string;
  success: boolean;
  error?: string;
  sizeBytes: number;
};

export type TrashResult = {
  operationId: number;
  outcomes: TrashItemOutcome[];
  successCount: number;
  failedCount: number;
  totalBytesTrashed: number;
};

export type CleanupCategorySummary = {
  category: string;
  label: string;
  fileCount: number;
  totalBytes: number;
  riskLevel: RiskLevel;
};

export type CleanupItem = {
  path: string;
  name: string;
  sizeBytes: number;
  modifiedAt?: string;
  category: string;
  categoryLabel: string;
  riskLevel: RiskLevel;
};

export type CleanupAnalysisResult = {
  rootsScanned: string[];
  truncated: boolean;
  items: CleanupItem[];
  categories: CleanupCategorySummary[];
};

export type ProposedMove = {
  sourcePath: string;
  destPath: string;
  fileName: string;
  category: string;
  categoryLabel: string;
  sizeBytes: number;
  conflict: boolean;
};

export type OrganizeProposalsResult = {
  scan?: ScanSummaryDto;
  rootPath: string;
  moveCount: number;
  totalBytes: number;
  moves: ProposedMove[];
};

export type MoveItemOutcome = {
  sourcePath: string;
  destPath: string;
  success: boolean;
  error?: string;
};

export type ApplyOrganizeResult = {
  operationId: number;
  outcomes: MoveItemOutcome[];
  successCount: number;
  failedCount: number;
};

export type OperationRecord = {
  id: number;
  operationType: string;
  payloadJson: string;
  recoverableBytes?: number;
  reversible: boolean;
  createdAt: string;
};

export type FileMetadata = {
  id?: number;
  path: string;
  name: string;
  extension?: string;
  sizeBytes: number;
  createdAt?: string;
  modifiedAt?: string;
  accessedAt?: string;
  mimeOrKind?: string;
  isSystemHint: boolean;
  contentHash?: string;
  category?: string;
  riskLevel?: RiskLevel;
};
