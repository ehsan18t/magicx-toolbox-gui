// Wire DTOs are generated from the Rust serde types by ts-rs during `cargo test` (./generated/); this module re-exports them.
// Only the tweak model is adapted to camelCase (frontend models, below); every other DTO stays as the wire sends it.

import type { IconName } from "$lib/design";
import type { AppPresence } from "./generated/AppPresence";
import type { AppStatusView } from "./generated/AppStatusView";
import type { AppView } from "./generated/AppView";
import type { ApplyOutcomeView as ApplyOutcome } from "./generated/ApplyOutcomeView";
import type { Attention } from "./generated/Attention";
import type { AttentionItem } from "./generated/AttentionItem";
import type { AttentionKind } from "./generated/AttentionKind";
import type { AttentionReason } from "./generated/AttentionReason";
import type { Availability } from "./generated/Availability";
import type { CategoryView as CategoryMeta } from "./generated/CategoryView";
import type { CpuInfo } from "./generated/CpuInfo";
import type { DeviceInfo } from "./generated/DeviceInfo";
import type { DiskInfo } from "./generated/DiskInfo";
import type { DownloadProgress } from "./generated/DownloadProgress";
import type { EffectAgreementView as EffectAgreement } from "./generated/EffectAgreementView";
import type { ElevationState } from "./generated/ElevationState";
import type { EntrySummary } from "./generated/EntrySummary";
import type { EntryValidity } from "./generated/EntryValidity";
import type { FirewallChangeView as FirewallChange } from "./generated/FirewallChangeView";
import type { FirewallDirection } from "./generated/FirewallDirection";
import type { FirewallOperation } from "./generated/FirewallOperation";
import type { FirewallProtocol } from "./generated/FirewallProtocol";
import type { FirewallRuleAction } from "./generated/FirewallRuleAction";
import type { GpuInfo } from "./generated/GpuInfo";
import type { HardwareInfo } from "./generated/HardwareInfo";
import type { LiveSystemInfo } from "./generated/LiveSystemInfo";
import type { MachineHardware } from "./generated/MachineHardware";
import type { HeldInfoView as HeldInfo } from "./generated/HeldInfoView";
import type { HostsAction } from "./generated/HostsAction";
import type { HostsChangeView as HostsChange } from "./generated/HostsChangeView";
import type { InstallRoute } from "./generated/InstallRoute";
import type { InvalidReason } from "./generated/InvalidReason";
import type { Level } from "./generated/Level";
import type { LogLevel } from "./generated/LogLevel";
import type { LogLineView as LogLine } from "./generated/LogLineView";
import type { LogSettingsView as LogSettings } from "./generated/LogSettingsView";
import type { LogSource } from "./generated/LogSource";
import type { LogTail } from "./generated/LogTail";
import type { ManualTestLogEvent } from "./generated/ManualTestLogEvent";
import type { ManualTestReport } from "./generated/ManualTestReport";
import type { ManualTestStatus } from "./generated/ManualTestStatus";
import type { ManualTestView as ManualTest } from "./generated/ManualTestView";
import type { MemoryInfo } from "./generated/MemoryInfo";
import type { MonitorInfo } from "./generated/MonitorInfo";
import type { MotherboardInfo } from "./generated/MotherboardInfo";
import type { NetworkInfo } from "./generated/NetworkInfo";
import type { ObservedStateView as ObservedState } from "./generated/ObservedStateView";
import type { OpFailureClass as FailureClass } from "./generated/OpFailureClass";
import type { RegistryAction } from "./generated/RegistryAction";
import type { RegistryChangeView as RegistryChange } from "./generated/RegistryChangeView";
import type { RegistryHive } from "./generated/RegistryHive";
import type { RegistryValue } from "./generated/RegistryValue";
import type { RegistryValueType } from "./generated/RegistryValueType";
import type { RestoreOutcomeView as RestoreOutcome } from "./generated/RestoreOutcomeView";
import type { RiskLevel as BackendRiskLevel } from "./generated/RiskLevel";
import type { SchedulerAction } from "./generated/SchedulerAction";
import type { SchedulerChangeView as SchedulerChange } from "./generated/SchedulerChangeView";
import type { ServiceChangeView as ServiceChange } from "./generated/ServiceChangeView";
import type { StartupView as ServiceStartupType } from "./generated/StartupView";
import type { SystemReading } from "./generated/SystemReading";
import type { TweakOptionView as TweakEffectOption } from "./generated/TweakOptionView";
import type { TweakStateView } from "./generated/TweakStateView";
import type { TweakStatusEvent } from "./generated/TweakStatusEvent";
import type { TweakStatusView } from "./generated/TweakStatusView";
import type { TweakView } from "./generated/TweakView";
import type { UnavailableOptView as UnavailableOpt } from "./generated/UnavailableOptView";
import type { UnknownCauseView as UnknownCause } from "./generated/UnknownCauseView";
import type { UnknownReasonView as UnknownReason } from "./generated/UnknownReasonView";
import type { UpdateConfig } from "./generated/UpdateConfig";
import type { UpdateInfo } from "./generated/UpdateInfo";
import type { WindowsInfo } from "./generated/WindowsInfo";

export type {
  ApplyOutcome,
  AppPresence,
  AppStatusView,
  AppView,
  Attention,
  AttentionItem,
  AttentionKind,
  AttentionReason,
  Availability,
  BackendRiskLevel,
  CategoryMeta,
  CpuInfo,
  DeviceInfo,
  DiskInfo,
  DownloadProgress,
  EffectAgreement,
  ElevationState,
  EntrySummary,
  EntryValidity,
  FailureClass,
  FirewallChange,
  FirewallDirection,
  FirewallOperation,
  FirewallProtocol,
  FirewallRuleAction,
  GpuInfo,
  HardwareInfo,
  LiveSystemInfo,
  MachineHardware,
  HeldInfo,
  HostsAction,
  HostsChange,
  InstallRoute,
  InvalidReason,
  Level,
  LogLevel,
  LogLine,
  LogSettings,
  LogSource,
  LogTail,
  ManualTest,
  ManualTestLogEvent,
  ManualTestReport,
  ManualTestStatus,
  MemoryInfo,
  MonitorInfo,
  MotherboardInfo,
  NetworkInfo,
  ObservedState,
  RegistryAction,
  RegistryChange,
  RegistryHive,
  RegistryValue,
  RegistryValueType,
  RestoreOutcome,
  SchedulerAction,
  SchedulerChange,
  ServiceChange,
  ServiceStartupType,
  SystemReading,
  TweakEffectOption,
  TweakStateView,
  TweakStatusEvent,
  TweakStatusView,
  TweakView,
  UnavailableOpt,
  UnknownCause,
  UnknownReason,
  UpdateConfig,
  UpdateInfo,
  WindowsInfo,
};

export type RiskLevel = Lowercase<BackendRiskLevel>;

export type ItemKind = "tweak" | "app";

/** `code` of a failed apply or restore (`Error::TweakFailed`), beside its unchanged `message`. */
export const TWEAK_FAILURE_CODES = [
  "TWEAK_ACCESS_DENIED",
  "TWEAK_NOT_FOUND",
  "TWEAK_BUSY",
  "TWEAK_ELEVATION_UNAVAILABLE",
  "TWEAK_OUTCOME_UNKNOWN",
  "TWEAK_VERIFY_MISMATCH",
  "TWEAK_ENGINE_ERROR",
] as const;
export type TweakFailureCode = (typeof TWEAK_FAILURE_CODES)[number];

/** `code` of every backend `Error` (src-tauri/src/error.rs). */
export const BACKEND_ERROR_CODES = [
  ...TWEAK_FAILURE_CODES,
  "APP_EXITING",
  "APP_FAILED",
  "APP_UNAVAILABLE",
  "APPLY_IN_FLIGHT",
  "BACKUP_FAILED",
  "COMMAND_EXECUTION_FAILED",
  "ELEVATION_DECLINED",
  "NOT_FOUND",
  "REGISTRY_ACCESS_DENIED",
  "REGISTRY_KEY_NOT_FOUND",
  "REGISTRY_OPERATION_FAILED",
  "REQUIRES_ADMIN",
  "SERVICE_CONTROL_FAILED",
  "TAURI_ERROR",
  "TWEAK_UNAVAILABLE",
  "UPDATE_ERROR",
  "UPDATE_FOLDER_READ_ONLY",
  "VALIDATION_FAILED",
  "WINDOWS_API_ERROR",
] as const;
export type BackendErrorCode = (typeof BACKEND_ERROR_CODES)[number];

/** What an app row offers; "store" opens the Store page rather than running here. */
export type AppActionKind = "remove" | "install" | "store";

export type AppOperationKind = Exclude<AppActionKind, "store">;

// Frontend models, adapted from the DTOs above.

export interface CategoryDefinition {
  id: string;
  name: string;
  description: string;
  icon: IconName;
  order: number;
}

export interface TweakDefinition {
  id: string;
  name: string;
  description: string;
  categoryId: string;
  riskLevel: RiskLevel;
  reversible: boolean;
  requiresReboot: boolean;
  /** The engine's run level, which the permission badge names (ADR-0005). */
  requiredLevel: Level;
  availability: Availability;
  supported: boolean;
  /** Apply targets are addressed by label, not index. */
  options: TweakEffectOption[];
  info: string | null;
  warning: string | null;
}

/** "loading" until the tweak's first `tweak-status` event arrives. */
export interface TweakStatus {
  state: "loading" | TweakStateView["state"];
  activeOption: string | null;
  unavailableReason: string | null;
  unknownReasons: UnknownReason[];
  /** Any Unknown reason is fixable by elevating. */
  needsElevation: boolean;
  unavailableOptions: UnavailableOpt[];
  residues: string[];
  heldShared: HeldInfo[];
  observed: ObservedState | null;
  hasHistory: boolean;
  attention: Attention | null;
}

export interface TweakWithStatus {
  definition: TweakDefinition;
  status: TweakStatus;
}

/** What the system card shows: the live fields over the hardware read. */
export type SystemInfo = LiveSystemInfo & MachineHardware;

/** The last good hardware read, shown while the next one runs. */
export type CachedSystemInfo = MachineHardware & { cachedAt: string };

export interface PendingChange {
  tweakId: string;
  optionLabel: string;
}

/** Persisted in localStorage. */
export interface AppSettings {
  autoCheckUpdates: boolean;
  /** ISO 8601. */
  lastUpdateCheck: string | null;
  /** List tweaks and apps this Windows build cannot run, shown as unavailable. */
  showUnsupported: boolean;
  includePrereleases: boolean;
}

// Logs panel and diagnostics (commands/logging.rs).

/** The levels the frontend forwards; the backend records anything else as info. */
export type ForwardedLogLevel = Extract<LogLevel, "error" | "warn" | "info">;

// Profiles: the planned shapes of a backend that does not exist yet (docs/spec/profile-v1.md).

export interface TweakSelection {
  tweak_id: string;
  selected_option_index: number;
  selected_option_label: string;
  option_content_hash?: string;
  category_id?: string;
}

export interface ProfileMetadata {
  name: string;
  description?: string;
  created_at: string;
  modified_at: string;
  app_version: string;
  source_windows_version: number;
  source_windows_build: number;
  source_machine_id?: string;
}

export interface RegistryValueState {
  hive: string;
  key: string;
  value_name: string;
  value_type?: string;
  value?: unknown;
  exists: boolean;
}

export interface ServiceState {
  name: string;
  startup_type: string;
  is_running: boolean;
  exists: boolean;
}

export interface SchedulerState {
  task_path: string;
  task_name: string;
  state: string;
  exists: boolean;
}

export interface SnapshotMetadata {
  created_at: string;
  app_version: string;
  windows_version: number;
  windows_build: number;
  machine_name: string;
}

export interface SystemStateSnapshot {
  schema_version: number;
  metadata: SnapshotMetadata;
  registry_state: RegistryValueState[];
  service_state: ServiceState[];
  scheduler_state: SchedulerState[];
}

export interface ConfigurationProfile {
  schema_version: number;
  metadata: ProfileMetadata;
  selections: TweakSelection[];
  system_state?: SystemStateSnapshot;
}

export type ProfileWarningCode =
  "WindowsVersionMismatch" | "TweakSchemaChanged" | "OptionResolvedByHash" | "TweakResolvedByAlias" | "AlreadyApplied";

export type ProfileErrorCode =
  | "SchemaVersionTooNew"
  | "TweakNotFound"
  | "WindowsVersionIncompatible"
  | "InvalidOptionIndex"
  | "ServiceNotFound"
  | "TaskNotFound";

export interface ValidationWarning {
  tweak_id: string;
  code: ProfileWarningCode;
  message: string;
}

export interface ValidationError {
  tweak_id: string;
  code: ProfileErrorCode;
  message: string;
}

export type ChangeType = "Registry" | "Service" | "ScheduledTask" | "Command";

export interface ChangeDetail {
  change_type: ChangeType;
  description: string;
  current_value?: string;
  new_value?: string;
}

export interface TweakChangePreview {
  tweak_id: string;
  tweak_name: string;
  category_id: string;
  current_option_index?: number;
  current_option_label?: string;
  target_option_index: number;
  target_option_label: string;
  applicable: boolean;
  skip_reason?: string;
  risk_level: BackendRiskLevel;
  already_applied: boolean;
  has_skipped_commands: boolean;
  changes: ChangeDetail[];
}

export interface ValidationStats {
  total_tweaks: number;
  applicable_tweaks: number;
  skipped_tweaks: number;
  already_applied: number;
  tweaks_with_warnings: number;
}

export interface ProfileValidation {
  is_valid: boolean;
  is_partially_applicable: boolean;
  warnings: ValidationWarning[];
  errors: ValidationError[];
  preview: TweakChangePreview[];
  stats: ValidationStats;
}

export interface ApplyFailure {
  tweak_id: string;
  tweak_name: string;
  error: string;
  was_rolled_back: boolean;
}

export interface ProfileApplyResult {
  success: boolean;
  applied_count: number;
  skipped_count: number;
  failed_count: number;
  failures: ApplyFailure[];
  requires_reboot: boolean;
  reboot_required_tweaks: string[];
}

export interface ExportOptions {
  description?: string;
  includeSystemState?: boolean;
}

export interface ApplyOptions {
  skipTweakIds?: string[];
  skipAlreadyApplied?: boolean;
  createRestorePoint?: boolean;
}
