/**
 * Profile API - Configuration profile export/import functionality
 *
 * This module provides functions for exporting and importing configuration profiles
 * to share tweak configurations across machines or backups.
 */

import { invoke } from "@tauri-apps/api/core";

// ============================================================================
// Types
// ============================================================================

/** Selection of a single tweak for inclusion in a profile */
export interface TweakSelection {
  tweak_id: string;
  selected_option_index: number;
  option_content_hash?: string;
}

/** Profile metadata */
export interface ProfileMetadata {
  profile_name: string;
  description?: string;
  created_at: string;
  app_version: string;
  source_machine_name?: string;
  source_windows_version: number;
  tags: string[];
}

/** Captured system state (optional) */
export interface ProfileSystemState {
  registry_state: Record<string, unknown>;
  services_state: Record<string, unknown>;
  tasks_state: Record<string, unknown>;
}

/** Full configuration profile */
export interface ConfigurationProfile {
  schema_version: number;
  metadata: ProfileMetadata;
  selections: TweakSelection[];
  system_state?: ProfileSystemState;
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

/** Result of applying a profile */
export interface ProfileApplyResult {
  success: boolean;
  applied_count: number;
  skipped_count: number;
  failed_tweaks: [string, string][];
  requires_reboot: boolean;
  backup_ids: string[];
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
  filePath: string,
  profileName: string,
  tweakIds: string[],
  options?: {
    description?: string;
    includeSystemState?: boolean;
  },
): Promise<void> {
  return invoke("profile_export", {
    filePath,
    name: profileName,
    description: options?.description ?? null,
    tweakIds,
    includeSystemState: options?.includeSystemState ?? false,
  });
}

/**
 * Import and validate a profile from a file.
 *
 * @param filePath - Path to the profile file
 * @returns The profile and validation result
 */
export async function importProfile(filePath: string): Promise<[ConfigurationProfile, ProfileValidation]> {
  return invoke("profile_import", { filePath });
}

/**
 * Validate a profile against current system.
 *
 * @param profile - The profile to validate
 * @returns Validation result
 */
export async function validateProfile(profile: ConfigurationProfile): Promise<ProfileValidation> {
  return invoke("profile_validate", { profile });
}

/**
 * Apply a validated profile to the system.
 *
 * @param profile - The profile to apply
 * @param options - Apply options
 * @returns Apply result
 */
export async function applyProfile(
  profile: ConfigurationProfile,
  options?: {
    skipTweakIds?: string[];
    skipAlreadyApplied?: boolean;
    createRestorePoint?: boolean;
  },
): Promise<ProfileApplyResult> {
  return invoke("profile_apply", {
    profile,
    skipTweakIds: options?.skipTweakIds ?? [],
    skipAlreadyApplied: options?.skipAlreadyApplied ?? true,
    createRestorePoint: options?.createRestorePoint ?? true,
  });
}

/**
 * Load a profile from file without validating.
 *
 * @param filePath - Path to the profile file
 * @returns The loaded profile
 */
export async function loadProfile(filePath: string): Promise<ConfigurationProfile> {
  return invoke("profile_load", { filePath });
}
