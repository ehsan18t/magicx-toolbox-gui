/**
 * Profile API - Configuration profile export/import functionality
 *
 * This module provides functions for exporting and importing configuration profiles
 * to share tweak configurations across machines or backups.
 */

// ============================================================================
// TEMPORARILY DISABLED
// ============================================================================
// The v1 profile backend was deleted in the 2026-07 cleanup and is being
// rebuilt from scratch. See docs/spec/profile-v1.md and docs/TWEAK_SYSTEM_PLAN.md.
//
// The UI is deliberately kept rather than removed, so the feature reads as
// "coming back" rather than "gone". Every backend call is neutralized here at a
// single choke point: the type definitions below have no backend dependency and
// still compile, and no `invoke()` reaches a command that no longer exists.
//
// To restore: reinstate the Tauri commands, then replace `profileBackendUnavailable()`
// with the original `invoke(...)` bodies (see git history for this file).
// ============================================================================

/** Message shown wherever a profile action is attempted while the backend is absent. */
export const PROFILE_UNAVAILABLE_MESSAGE = "Profiles are temporarily unavailable while the system is rebuilt.";

function profileBackendUnavailable(): Promise<never> {
  return Promise.reject(new Error(PROFILE_UNAVAILABLE_MESSAGE));
}

// ============================================================================
// Types
// ============================================================================

/** Selection of a single tweak for inclusion in a profile */
export interface TweakSelection {
  tweak_id: string;
  selected_option_index: number;
  selected_option_label: string;
  option_content_hash?: string;
  category_id?: string;
}

/** Profile metadata - matches backend ProfileMetadata struct */
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

/** State of a single registry value */
export interface RegistryValueState {
  hive: string;
  key: string;
  value_name: string;
  value_type?: string;
  value?: unknown;
  exists: boolean;
}

/** State of a Windows service */
export interface ServiceState {
  name: string;
  startup_type: string;
  is_running: boolean;
  exists: boolean;
}

/** State of a scheduled task */
export interface SchedulerState {
  task_path: string;
  task_name: string;
  state: string;
  exists: boolean;
}

/** Snapshot metadata */
export interface SnapshotMetadata {
  created_at: string;
  app_version: string;
  windows_version: number;
  windows_build: number;
  machine_name: string;
}

/** System state snapshot - matches backend SystemStateSnapshot struct */
export interface SystemStateSnapshot {
  schema_version: number;
  metadata: SnapshotMetadata;
  registry_state: RegistryValueState[];
  service_state: ServiceState[];
  scheduler_state: SchedulerState[];
}

/** Full configuration profile */
export interface ConfigurationProfile {
  schema_version: number;
  metadata: ProfileMetadata;
  selections: TweakSelection[];
  system_state?: SystemStateSnapshot;
}

/** Warning code for validation warnings */
export type WarningCode =
  | "WindowsVersionMismatch"
  | "TweakSchemaChanged"
  | "OptionResolvedByHash"
  | "TweakResolvedByAlias"
  | "AlreadyApplied";

/** Error code for validation errors */
export type ErrorCode =
  | "SchemaVersionTooNew"
  | "TweakNotFound"
  | "WindowsVersionIncompatible"
  | "InvalidOptionIndex"
  | "ServiceNotFound"
  | "TaskNotFound";

/** Validation warning */
export interface ValidationWarning {
  tweak_id: string;
  code: WarningCode;
  message: string;
}

/** Validation error */
export interface ValidationError {
  tweak_id: string;
  code: ErrorCode;
  message: string;
}

/** Change type for preview */
export type ChangeType = "Registry" | "Service" | "ScheduledTask" | "Command";

/** Change detail for preview */
export interface ChangeDetail {
  change_type: ChangeType;
  description: string;
  current_value?: string;
  new_value?: string;
}

/** Preview of changes for a single tweak */
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
  /** Whether this tweak has commands that will be skipped during profile apply */
  has_skipped_commands: boolean;
  changes: ChangeDetail[];
}

/** Validation statistics */
export interface ValidationStats {
  total_tweaks: number;
  applicable_tweaks: number;
  skipped_tweaks: number;
  already_applied: number;
  tweaks_with_warnings: number;
}

/** Full validation result */
export interface ProfileValidation {
  is_valid: boolean;
  is_partially_applicable: boolean;
  warnings: ValidationWarning[];
  errors: ValidationError[];
  preview: TweakChangePreview[];
  stats: ValidationStats;
}

/** Details of a failed tweak application */
export interface ApplyFailure {
  tweak_id: string;
  tweak_name: string;
  error: string;
  was_rolled_back: boolean;
}

/** Result of applying a profile */
export interface ProfileApplyResult {
  success: boolean;
  applied_count: number;
  skipped_count: number;
  failed_count: number;
  failures: ApplyFailure[];
  requires_reboot: boolean;
  reboot_required_tweaks: string[];
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Export a configuration profile to a file.
 *
 * @param filePath - Path to save the profile
 * @param profileName - Name for the profile
 * @param tweakIds - Array of tweak IDs to include in the profile
 * @param options - Optional export options
 */
export async function exportProfile(
  _filePath: string,
  _profileName: string,
  _tweakIds: string[],
  _options?: {
    description?: string;
    includeSystemState?: boolean;
  },
): Promise<void> {
  return profileBackendUnavailable();
}

/**
 * Import and validate a profile from a file.
 *
 * @param filePath - Path to the profile file
 * @returns The profile and validation result
 */
export async function importProfile(_filePath: string): Promise<[ConfigurationProfile, ProfileValidation]> {
  return profileBackendUnavailable();
}

/**
 * Validate a profile against current system.
 *
 * @param profile - The profile to validate
 * @returns Validation result
 */
export async function validateProfile(_profile: ConfigurationProfile): Promise<ProfileValidation> {
  return profileBackendUnavailable();
}

/**
 * Apply a validated profile to the system.
 *
 * @param profile - The profile to apply
 * @param options - Apply options
 * @returns Apply result
 */
export async function applyProfile(
  _profile: ConfigurationProfile,
  _options?: {
    skipTweakIds?: string[];
    skipAlreadyApplied?: boolean;
    createRestorePoint?: boolean;
  },
): Promise<ProfileApplyResult> {
  return profileBackendUnavailable();
}

/**
 * Get list of saved profiles from the app data directory or a custom path.
 */
export async function getSavedProfiles(_customPath?: string | null): Promise<ProfileMetadata[]> {
  return profileBackendUnavailable();
}

/**
 * Delete a saved profile by name.
 * @param name Profile name (without extension)
 * @param customPath Optional custom directory path
 */
export async function deleteSavedProfile(_name: string, _customPath?: string | null): Promise<void> {
  return profileBackendUnavailable();
}
