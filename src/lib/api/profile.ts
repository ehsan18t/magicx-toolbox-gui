// The profile backend does not exist yet (docs/spec/profile-v1.md). The UI stays, so the feature reads as
// coming back; every call rejects here, at one choke point, and no invoke() reaches an unregistered command.

export const PROFILE_UNAVAILABLE_MESSAGE = "Profiles are temporarily unavailable while the system is rebuilt.";

function profileBackendUnavailable(): Promise<never> {
  return Promise.reject(new Error(PROFILE_UNAVAILABLE_MESSAGE));
}

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

export type WarningCode =
  "WindowsVersionMismatch" | "TweakSchemaChanged" | "OptionResolvedByHash" | "TweakResolvedByAlias" | "AlreadyApplied";

export type ErrorCode =
  | "SchemaVersionTooNew"
  | "TweakNotFound"
  | "WindowsVersionIncompatible"
  | "InvalidOptionIndex"
  | "ServiceNotFound"
  | "TaskNotFound";

export interface ValidationWarning {
  tweak_id: string;
  code: WarningCode;
  message: string;
}

export interface ValidationError {
  tweak_id: string;
  code: ErrorCode;
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
  risk_level: string;
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

export async function exportProfile(
  _filePath: string,
  _profileName: string,
  _tweakIds: string[],
  _options?: ExportOptions,
): Promise<void> {
  return profileBackendUnavailable();
}

/** Loads and validates the profile at the path. */
export async function importProfile(_filePath: string): Promise<[ConfigurationProfile, ProfileValidation]> {
  return profileBackendUnavailable();
}

export async function applyProfile(
  _profile: ConfigurationProfile,
  _options?: ApplyOptions,
): Promise<ProfileApplyResult> {
  return profileBackendUnavailable();
}

/** From the app data directory, or `customPath` when given. */
export async function getSavedProfiles(_customPath?: string | null): Promise<ProfileMetadata[]> {
  return profileBackendUnavailable();
}

/** `name` without the extension. */
export async function deleteSavedProfile(_name: string, _customPath?: string | null): Promise<void> {
  return profileBackendUnavailable();
}
