// Wire DTOs mirror the serde shapes from src-tauri/src/commands/*.rs and live here; api/ only wraps commands.
// Only the tweak model is adapted to camelCase (frontend models, below); every other DTO stays as the wire sends it.

import type { IconName } from "$lib/components/shared";

export type RegistryHive = "HKCU" | "HKLM";

export type RegistryValueType = "REG_DWORD" | "REG_SZ" | "REG_EXPAND_SZ" | "REG_BINARY" | "REG_MULTI_SZ" | "REG_QWORD";

export type ServiceStartupType = "disabled" | "manual" | "automatic" | "automatic_delayed" | "boot" | "system";

/** REG_MULTI_SZ is string[], REG_BINARY number[] (bytes). */
export type RegistryValue = number | string | string[] | number[] | null;

export type RegistryAction = "set" | "delete_value" | "delete_key" | "create_key";

export interface RegistryChange {
  hive: RegistryHive;
  key: string;
  value_name: string;
  action: RegistryAction;
  /** Null for the delete and create-key actions. */
  value_type: RegistryValueType | null;
  value: RegistryValue;
  /** Empty or absent applies to every version. */
  windows_versions?: number[];
  skip_validation?: boolean;
}

export interface ServiceChange {
  name: string;
  startup: ServiceStartupType;
  skip_validation?: boolean;
}

export type SchedulerAction = "enable" | "disable";

export interface SchedulerChange {
  task_path: string;
  action: SchedulerAction;
  skip_validation?: boolean;
}

export type HostsAction = "add" | "remove";

export interface HostsChange {
  ip: string;
  domain: string;
  action: HostsAction;
  comment?: string;
  skip_validation?: boolean;
}

export type FirewallDirection = "inbound" | "outbound";

export type FirewallRuleAction = "block" | "allow";

export type FirewallProtocol = "any" | "tcp" | "udp" | "icmpv4" | "icmpv6";

export type FirewallOperation = "create" | "delete";

export interface FirewallChange {
  name: string;
  operation: FirewallOperation;
  /** Required for create. */
  direction?: FirewallDirection;
  /** Required for create. */
  action?: FirewallRuleAction;
  protocol?: FirewallProtocol;
  program?: string;
  service?: string;
  remote_addresses?: string[];
  remote_ports?: string;
  local_ports?: string;
  description?: string;
  skip_validation?: boolean;
}

/** Elevation floor / app ceiling (serde: exact Rust variant names). */
export type Level = "User" | "Admin" | "Ti";

export type BackendRiskLevel = "Low" | "Medium" | "High" | "Critical";

export type RiskLevel = Lowercase<BackendRiskLevel>;

export type ItemKind = "tweak" | "app";

// sid_mismatch identified another account, sid_unknown read no SID: only the first may name another account.
// elevation_path_unavailable (TrustedInstaller off) is not fixed by restarting elevated, so it never says to.
export type Availability =
  | { state: "available" }
  | { state: "needs_elevation"; reason: string }
  | { state: "sid_mismatch"; reason: string }
  | { state: "sid_unknown"; reason: string }
  | { state: "elevation_path_unavailable"; reason: string };

export interface CategoryMeta {
  id: string;
  name: string;
  icon: string;
  description: string;
}

/** One option with the concrete per-effect changes it drives. */
export interface TweakEffectOption {
  label: string;
  registry_changes: RegistryChange[];
  service_changes: ServiceChange[];
  scheduler_changes: SchedulerChange[];
  hosts_changes: HostsChange[];
  firewall_changes: FirewallChange[];
  /** Action scripts, shown verbatim. */
  commands: string[];
}

export interface TweakView {
  id: string;
  name: string;
  description: string;
  /** Markdown for the Details modal. */
  info: string | null;
  warning: string | null;
  category: string;
  risk: BackendRiskLevel;
  reversible: boolean;
  requires_reboot: boolean;
  /** Apply targets are addressed by label. */
  options: TweakEffectOption[];
  /** The level the engine actually runs the tweak at, not its declared floor (ADR-0005). */
  required_level: Level;
  availability: Availability;
  /** False when this Windows build can run none of its effects. */
  supported: boolean;
}

export type UnknownCause = "AccessDenied" | "Malformed" | "MissingRequired" | "Other";

export interface UnknownReason {
  effect: string;
  cause: UnknownCause;
  /** True only when elevating could resolve the read (AccessDenied). */
  needs_elevation: boolean;
}

export interface UnavailableOpt {
  label: string;
  reason: string;
}

/** Which authored options wanted the value one effect actually holds. Empty = none do. */
export interface EffectAgreement {
  effect: string;
  /** The registry value / service / task the effect addresses, not the author's effect id. */
  name: string;
  wanted_by: string[];
}

/** What the machine reads when it matches no option, shaped like one so it renders beside them. */
export interface ObservedState {
  changes: TweakEffectOption;
  agreement: EffectAgreement[];
}

export interface HeldInfo {
  shared: string;
  holders: string[];
}

export type TweakStateView =
  | { state: "active"; option: string }
  | { state: "system_default" }
  | { state: "unavailable"; reason: string }
  | { state: "unknown"; reasons: UnknownReason[] };

/** The step kind that could not be verified, so the UI never parses a message to find out. */
export type AttentionKind =
  | "drive"
  | "verify"
  | "outcome_unknown"
  | "action"
  | "no_undo"
  | "claim"
  | "store"
  | "crash_residue"
  | "unrecorded"
  | "other";

/** serde: `OpFailureClass`. */
export type FailureClass = "access_denied" | "not_found" | "invalid_data" | "busy" | "failed";

export interface AttentionItem {
  effect: string | null;
  kind: AttentionKind;
  /** Absent when the failure was not classified, or the record predates classes. */
  class?: FailureClass;
  /** Snapshot entry seqs whose open drive mark this item accounts for. */
  entries?: number[];
  message: string;
}

/** `code` of a failed apply or restore (`Error::TweakFailed`), beside its unchanged `message`. */
export type TweakFailureCode =
  | "TWEAK_ACCESS_DENIED"
  | "TWEAK_NOT_FOUND"
  | "TWEAK_BUSY"
  | "TWEAK_ELEVATION_UNAVAILABLE"
  | "TWEAK_OUTCOME_UNKNOWN"
  | "TWEAK_VERIFY_MISMATCH"
  | "TWEAK_ENGINE_ERROR";

/** `code` of every backend `Error` (src-tauri/src/error.rs). */
export type BackendErrorCode =
  | TweakFailureCode
  | "APP_EXITING"
  | "APP_FAILED"
  | "APP_UNAVAILABLE"
  | "APPLY_IN_FLIGHT"
  | "BACKUP_FAILED"
  | "COMMAND_EXECUTION_FAILED"
  | "NOT_FOUND"
  | "REGISTRY_ACCESS_DENIED"
  | "REGISTRY_KEY_NOT_FOUND"
  | "REGISTRY_OPERATION_FAILED"
  | "REQUIRES_ADMIN"
  | "SERVICE_CONTROL_FAILED"
  | "TAURI_ERROR"
  | "TWEAK_UNAVAILABLE"
  | "UPDATE_ERROR"
  | "VALIDATION_FAILED"
  | "WINDOWS_API_ERROR";

/** Kept per tweak, not per snapshot entry, so releasing an entry cannot drop it (ADR-0001/0002). */
export type AttentionReason =
  "apply_failed" | "restore_failed" | "crash_residue" | "outcome_unrecorded" | "record_unreadable";

export interface Attention {
  reason: AttentionReason;
  items: AttentionItem[];
}

export interface TweakStatusView {
  state: TweakStateView;
  unavailable: UnavailableOpt[];
  residues: string[];
  has_history: boolean;
  attention: Attention | null;
  /** Publication order: a status stamped lower than the one already shown read the machine earlier. */
  stamp: number;
  held_shared: HeldInfo[];
  /** Non-null only at System Default. */
  observed: ObservedState | null;
}

export interface TweakStatusEvent {
  tweak_id: string;
  status: TweakStatusView;
}

/** `sid_mismatch` also covers an unreadable SID; only `Availability` tells them apart, so never render it as another account. */
export interface ElevationState {
  level: Level;
  sid_mismatch: boolean;
}

export interface ApplyOutcome {
  effects: { effect: string; kind: Record<string, unknown> }[];
  status: TweakStatusView;
}

export type InvalidReason =
  "Corrupt" | "WrongSchema" | "WrongMachine" | "WrongUser" | "DanglingRef" | "TargetUnavailable";

/** Externally tagged. */
export type EntryValidity = "Valid" | { Invalid: InvalidReason };

export interface EntrySummary {
  seq: number;
  validity: EntryValidity;
  timestamp: string | null;
  captured: unknown;
}

export interface RestoreOutcome {
  status: TweakStatusView;
  consumed: number | null;
  reboot_advisory: boolean;
  skipped_invalid: EntrySummary[];
}

/** Not a tweak: no options, no snapshot (ADR-0009). */
export interface AppView {
  id: string;
  name: string;
  description: string;
  info: string | null;
  warning: string | null;
  category: string;
  risk: BackendRiskLevel;
  /** `appx` removes for every account; `script` acts on the running account. */
  source: "appx" | "script";
  /** As authored; `InstallRoute` is the route this machine can actually use. */
  install: { kind: "store" | "winget" | "store_page"; id: string } | null;
  remove_availability: Availability;
  install_availability: Availability;
  supported: boolean;
}

/** Unknown is never Absent: an unreadable app stays visible with its actions disabled. */
export type AppPresence =
  | { state: "installed"; provisioned_only: boolean }
  | { state: "absent" }
  | { state: "unknown"; reason: string; needs_elevation: boolean };

/** "none" makes a removal permanent. */
export type InstallRoute = "winget" | "store_page" | "none";

/** What an app row offers; "store" opens the Store page rather than running here. */
export type AppActionKind = "remove" | "install" | "store";

export type AppOperationKind = Exclude<AppActionKind, "store">;

export interface AppStatusView {
  app_id: string;
  presence: AppPresence;
  install_route: InstallRoute;
  stamp: number;
}

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
  info?: string;
  warning?: string;
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

export interface WindowsInfo {
  product_name: string;
  display_version: string;
  build_number: string;
  is_windows_11: boolean;
  /** "10" or "11". */
  version_string: string;
  is_windows_server: boolean;
  uptime_seconds: number;
  /** ISO 8601. */
  install_date: string | null;
}

export interface DeviceInfo {
  manufacturer: string;
  model: string;
  system_type: string;
  /** Desktop, Laptop, Workstation, … */
  pc_type: string;
}

export interface CpuInfo {
  name: string;
  cores: number;
  threads: number;
  architecture: string;
  max_clock_mhz: number;
}

export interface GpuInfo {
  name: string;
  memory_gb: number;
  driver_version: string;
  processor: string;
  refresh_rate: number;
  /** Resolution and colour depth. */
  video_mode: string;
}

export interface DiskInfo {
  model: string;
  size_gb: number;
  drive_type: string;
  interface_type: string;
  health_status: string | null;
}

export interface MemoryInfo {
  total_gb: number;
  speed_mhz: number;
  memory_type: string;
  slots_used: number;
}

export interface MotherboardInfo {
  manufacturer: string;
  product: string;
  bios_version: string;
}

export interface NetworkInfo {
  name: string;
  mac_address: string;
  ip_address: string;
  dhcp_enabled: boolean;
}

export interface MonitorInfo {
  name: string;
  resolution: string;
  refresh_rate: number;
}

export interface HardwareInfo {
  cpu: CpuInfo;
  gpu: GpuInfo[];
  monitors: MonitorInfo[];
  memory: MemoryInfo;
  motherboard: MotherboardInfo;
  disks: DiskInfo[];
  network: NetworkInfo[];
  total_storage_gb: number;
}

export interface SystemInfo {
  windows: WindowsInfo;
  computer_name: string;
  username: string;
  is_admin: boolean;
  hardware: HardwareInfo;
  device: DeviceInfo;
}

/** The static part of SystemInfo; uptime and elevation are re-read on every load. */
export type CachedSystemInfo = Pick<SystemInfo, "hardware" | "device" | "computer_name"> & { cachedAt: string };

export interface PendingChange {
  tweakId: string;
  optionLabel: string;
}

/** Persisted in localStorage. */
export interface AppSettings {
  autoCheckUpdates: boolean;
  autoInstallUpdates: boolean;
  /** ISO 8601. */
  lastUpdateCheck: string | null;
  /** List tweaks and apps this Windows build cannot run, shown as unavailable. */
  showUnsupported: boolean;
  includePrereleases: boolean;
}

export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion?: string;
  releaseNotes?: string;
  downloadUrl?: string;
  publishedAt?: string;
  assetName?: string;
  /** Bytes. */
  assetSize?: number;
  /** GitHub's `sha256:<hex>`, required to install. */
  assetDigest?: string;
  prerelease: boolean;
}

// Logs panel and diagnostics (commands/logging.rs).

export type LogLevel = "error" | "warn" | "info" | "debug" | "trace";

/** The levels the frontend forwards; the backend records anything else as info. */
export type ForwardedLogLevel = Extract<LogLevel, "error" | "warn" | "info">;

export type LogSource = "app" | "ui" | "helper";

export interface LogLine {
  seq: number;
  ts: string;
  level: LogLevel;
  source: LogSource;
  target: string;
  msg: string;
}

export interface LogTail {
  lines: LogLine[];
  /** Lines after `since` that left the in-memory buffer before they could be read. */
  skipped: number;
}

export interface LogSettings {
  persist: boolean;
  detailed: boolean;
  folder: string;
  writing: boolean;
  error: string | null;
  files: number;
  bytes: number;
}

// Manual tests, test build only (commands/manual_tests.rs).

export type ManualTestStatus = "pass" | "fail" | "info";

export interface ManualTest {
  id: string;
  title: string;
  description: string;
  changes: string;
  changes_system: boolean;
  /** Default duration in minutes, for a test that runs over time. */
  minutes: number | null;
}

export interface ManualTestReport {
  test_id: string;
  status: ManualTestStatus;
  summary: string;
  details: string[];
  report: string;
}

export interface ManualTestLogEvent {
  test_id: string;
  line: string;
}

// App updates (commands/update.rs).

export interface UpdateConfig {
  releasesApiUrl: string;
  /** regex_lite syntax. */
  assetPattern: string;
  includePrereleases: boolean;
}

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
