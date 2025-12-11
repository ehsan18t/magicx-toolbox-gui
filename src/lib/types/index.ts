// Type definitions matching Rust models

/** Risk level for tweaks */
export type RiskLevel = "low" | "medium" | "high" | "critical";

/** Registry hive types */
export type RegistryHive = "HKCU" | "HKLM";

/** Registry value types */
export type RegistryValueType =
  | "REG_DWORD"
  | "REG_SZ"
  | "REG_EXPAND_SZ"
  | "REG_BINARY"
  | "REG_MULTI_SZ"
  | "REG_QWORD";

/** Windows service startup type */
export type ServiceStartupType = "disabled" | "manual" | "automatic" | "boot" | "system";

/** Service change for a specific option (simplified: just target state) */
export interface OptionServiceChange {
  /** Service name (e.g., "SysMain", "DiagTrack") */
  name: string;
  /** Target startup type when this option is selected */
  startup: ServiceStartupType;
  /** Stop the service if startup is disabled */
  stop_if_disabled?: boolean;
}

/** Option for multi-state tweaks (displayed as dropdown) */
export interface TweakOption {
  label: string;
  value: unknown;
  is_default?: boolean;
  /** Service changes specific to this option */
  service_changes?: OptionServiceChange[];
}

/** Single registry change operation */
export interface RegistryChange {
  hive: RegistryHive;
  key: string;
  value_name: string;
  value_type: RegistryValueType;
  enable_value: unknown;
  disable_value?: unknown;
  /** Optional Windows version filter. If undefined/empty, applies to all versions. */
  windows_versions?: number[];
  /** Multi-state options (if present, displayed as dropdown instead of toggle) */
  options?: TweakOption[];
}

/** Single service change operation */
export interface ServiceChange {
  /** Service name (e.g., "wuauserv" for Windows Update) */
  name: string;
  /** Startup type when tweak is enabled (applied) */
  enable_startup: ServiceStartupType;
  /** Startup type when tweak is disabled (reverted) */
  disable_startup: ServiceStartupType;
  /** Stop the service when applying the tweak */
  stop_on_disable?: boolean;
  /** Start the service when reverting the tweak */
  start_on_enable?: boolean;
}

/** Category definition loaded from YAML file */
export interface CategoryDefinition {
  id: string;
  name: string;
  description: string;
  /** Iconify icon name (e.g., 'mdi:shield-lock') */
  icon: string;
  order: number;
}

/** A complete tweak definition loaded from YAML */
export interface TweakDefinition {
  id: string;
  name: string;
  description: string;
  category: string; // Dynamic category ID from YAML
  risk_level: RiskLevel;
  requires_reboot: boolean;
  requires_admin: boolean;
  /** List of registry changes (with optional windows_versions filter on each) */
  registry_changes: RegistryChange[];
  /** List of Windows service changes (start/stop, enable/disable) */
  service_changes?: ServiceChange[];
  /** Additional info/documentation */
  info?: string;
}

/** Status of a tweak in the system */
export interface TweakStatus {
  tweak_id: string;
  is_applied: boolean;
  last_applied?: string; // ISO 8601 timestamp
  has_backup: boolean;
  /** Current selected option index for multi-state tweaks */
  current_option_index?: number;
}

/** Combined tweak info for UI display */
export interface TweakWithStatus {
  definition: TweakDefinition;
  status: TweakStatus;
}

/** Windows system information */
export interface WindowsInfo {
  product_name: string;
  display_version: string;
  build_number: string;
  is_windows_11: boolean;
  version_string: string; // "10" or "11"
}

/** System information */
export interface SystemInfo {
  windows: WindowsInfo;
  computer_name: string;
  username: string;
  is_admin: boolean;
}

/** Result of applying a tweak */
export interface TweakResult {
  success: boolean;
  message: string;
  requires_reboot: boolean;
}

/** Batch apply result */
export interface BatchApplyResult {
  success: boolean;
  results: Record<string, TweakResult>;
  total_applied: number;
  total_failed: number;
}

/** Pending change for staged apply pattern */
export type PendingChange =
  | { type: "binary"; enabled: boolean }
  | { type: "multistate"; optionIndex: number };

/**
 * UI display information for risk levels.
 * These are presentation-layer constants for displaying risk level metadata to users.
 * The risk level identifiers (low, medium, high, critical) must match the backend RiskLevel enum.
 * These descriptions are intentionally kept in the frontend as they are purely for UI display
 * and do not affect any backend logic or tweak behavior.
 */
export const RISK_INFO: Record<RiskLevel, { name: string; description: string }> = {
  low: {
    name: "Low",
    description: "Safe to apply/revert without issues",
  },
  medium: {
    name: "Medium",
    description: "May require restart or have minor side effects",
  },
  high: {
    name: "High",
    description: "Could significantly impact system",
  },
  critical: {
    name: "Critical",
    description: "Could break Windows, use with caution",
  },
};
