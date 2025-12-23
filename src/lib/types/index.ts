// Type definitions matching Rust models

/** Risk level for tweaks */
export type RiskLevel = "low" | "medium" | "high" | "critical";

/** Permission level for tweaks (hierarchical: ti > system > admin > none) */
export type PermissionLevel = "none" | "admin" | "system" | "ti";

/** Registry hive types */
export type RegistryHive = "HKCU" | "HKLM";

/** Registry value types */
export type RegistryValueType = "REG_DWORD" | "REG_SZ" | "REG_EXPAND_SZ" | "REG_BINARY" | "REG_MULTI_SZ" | "REG_QWORD";

/** Windows service startup type */
export type ServiceStartupType = "disabled" | "manual" | "automatic" | "boot" | "system";

/**
 * Registry value type - maps to the RegistryValueType enum.
 * - REG_DWORD: 32-bit number (0 to 4294967295)
 * - REG_QWORD: 64-bit number
 * - REG_SZ: String
 * - REG_EXPAND_SZ: Expandable string (contains environment variables)
 * - REG_MULTI_SZ: Array of strings
 * - REG_BINARY: Array of numbers (bytes)
 */
export type RegistryValue = number | string | string[] | number[] | null;

// ============================================================================
// NEW UNIFIED OPTION-BASED TWEAK SYSTEM
// ============================================================================

/** Action type for registry operations */
export type RegistryAction = "set" | "delete_value" | "delete_key" | "create_key";

/** Registry change within an option */
export interface RegistryChange {
  hive: RegistryHive;
  key: string;
  value_name: string;
  /** Action to perform: set value, delete value, delete key, or create key */
  action: RegistryAction;
  /** Value type (required for set action, null for delete operations) */
  value_type: RegistryValueType | null;
  /** The value to set when this option is selected (null for delete operations) */
  value: RegistryValue;
  /** Optional Windows version filter. If undefined/empty, applies to all versions. */
  windows_versions?: number[];
  /** If true, skip this change for tweak status validation and ignore failures during apply */
  skip_validation?: boolean;
}

/** Service change within an option */
export interface ServiceChange {
  /** Service name (e.g., "SysMain", "DiagTrack") */
  name: string;
  /** Target startup type when this option is selected */
  startup: ServiceStartupType;
  /** If true, skip this change for tweak status validation and ignore failures during apply */
  skip_validation?: boolean;
}

/** Action for scheduled task changes */
export type SchedulerAction = "enable" | "disable" | "delete";

/** Scheduler change within an option */
export interface SchedulerChange {
  /** Task path in Task Scheduler (e.g., "\\Microsoft\\Windows\\Application Experience") */
  task_path: string;
  /** Exact task name (e.g., "Microsoft Compatibility Appraiser"). Mutually exclusive with task_name_pattern. */
  task_name?: string;
  /** Regex pattern to match multiple task names (e.g., "USO|Reboot|Refresh"). Mutually exclusive with task_name. */
  task_name_pattern?: string;
  /** Action to perform on the task(s) */
  action: SchedulerAction;
  /** If true, skip this change for tweak status validation and ignore failures during apply */
  skip_validation?: boolean;
  /** If true, don't error if task/path not found (useful for optional tasks) */
  ignore_not_found?: boolean;
}

/** Action for hosts file changes */
export type HostsAction = "add" | "remove";

/** Hosts file change within an option */
export interface HostsChange {
  /** IP address to map (e.g., "127.0.0.1", "0.0.0.0") */
  ip: string;
  /** Domain/hostname to block or redirect (e.g., "telemetry.microsoft.com") */
  domain: string;
  /** Action to perform: add or remove */
  action: HostsAction;
  /** Optional comment to add after the entry */
  comment?: string;
  /** If true, skip this change for tweak status validation */
  skip_validation?: boolean;
}

/** Direction for firewall rules */
export type FirewallDirection = "inbound" | "outbound";

/** Action for firewall rules */
export type FirewallRuleAction = "block" | "allow";

/** Protocol for firewall rules */
export type FirewallProtocol = "any" | "tcp" | "udp" | "icmpv4" | "icmpv6";

/** Firewall operation type */
export type FirewallOperation = "create" | "delete";

/** Firewall rule change within an option */
export interface FirewallChange {
  /** Unique rule name (e.g., "Block DiagTrack Telemetry") */
  name: string;
  /** Operation to perform: create or delete */
  operation: FirewallOperation;
  /** Direction: inbound or outbound (required for create) */
  direction?: FirewallDirection;
  /** Action: block or allow (required for create) */
  action?: FirewallRuleAction;
  /** Protocol to match (defaults to any) */
  protocol?: FirewallProtocol;
  /** Program/executable path to match */
  program?: string;
  /** Service name to match */
  service?: string;
  /** Remote addresses to match (e.g., ["157.56.0.0/16"]) */
  remote_addresses?: string[];
  /** Remote ports to match (e.g., "80,443") */
  remote_ports?: string;
  /** Local ports to match */
  local_ports?: string;
  /** Description for the rule */
  description?: string;
  /** If true, skip this change for tweak status validation */
  skip_validation?: boolean;
}

/** A single option within a tweak - contains all changes for that state */
export interface TweakOption {
  /** Display label (e.g., "Enabled", "Disabled", "4MB") */
  label: string;
  /** Registry modifications for this option */
  registry_changes: RegistryChange[];
  /** Service modifications for this option */
  service_changes: ServiceChange[];
  /** Scheduler task modifications for this option */
  scheduler_changes: SchedulerChange[];
  /** Hosts file modifications for this option */
  hosts_changes: HostsChange[];
  /** Firewall rule modifications for this option */
  firewall_changes: FirewallChange[];
  /** Shell commands to run BEFORE applying changes */
  pre_commands: string[];
  /** PowerShell commands to run BEFORE applying changes (after pre_commands) */
  pre_powershell: string[];
  /** Shell commands to run AFTER applying changes */
  post_commands: string[];
  /** PowerShell commands to run AFTER applying changes (after post_commands) */
  post_powershell: string[];
  /**
   * If true, treat missing registry keys/values as matching this option.
   * Used for tweaks that modify registry entries which may not exist on all Windows editions.
   * When a registry entry doesn't exist and this flag is set, the status is inferred rather than detected.
   */
  registry_missing_is_match?: boolean;
  /**
   * If true, treat missing services as matching this option.
   * Used for tweaks that disable services which may not exist on all Windows editions.
   * When a service doesn't exist and this flag is set, the status is inferred rather than detected.
   */
  service_missing_is_match?: boolean;
  /**
   * If true, treat missing scheduled tasks as matching this option.
   * Used for tweaks that disable tasks which may not exist on all Windows editions.
   * When a task doesn't exist and this flag is set, the status is inferred rather than detected.
   */
  scheduler_missing_is_match?: boolean;
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
  /** Category ID from YAML */
  category_id: string;
  risk_level: RiskLevel;
  requires_reboot: boolean;
  requires_admin: boolean;
  /** Requires SYSTEM elevation for protected registry keys */
  requires_system: boolean;
  /** Requires TrustedInstaller elevation for protected services (e.g., WaaSMedicSvc) */
  requires_ti: boolean;
  /** Additional info/documentation */
  info?: string;
  /** Force dropdown UI even with 2 options (default: false). 2 options = toggle, 3+ = dropdown */
  force_dropdown: boolean;
  /** Available options for this tweak (minimum 2) */
  options: TweakOption[];
}

/** Status of a tweak in the system */
export interface TweakStatus {
  tweak_id: string;
  /** Whether we have a snapshot (tweak was applied by this app) */
  is_applied: boolean;
  /** When the tweak was last applied (ISO 8601 timestamp) */
  last_applied?: string;
  /** Whether a snapshot exists for reverting */
  has_backup: boolean;
  /** Current option index that matches system state, or null if "System Default" */
  current_option_index: number | null;
  /**
   * The original option index from the snapshot, if one exists.
   * - undefined: No snapshot exists (tweak was never applied)
   * - null: Snapshot exists but original state was unknown (didn't match any option)
   * - number: Snapshot exists and original state matched that option index
   * Used by frontend to show "Default" segment when original state was unknown.
   */
  snapshot_original_option_index?: number | null;
  /**
   * True if the status was inferred from missing items (via missing_is_match flag)
   * rather than detected from actual registry/service values.
   * Used by frontend to show an indicator that the status is based on missing components.
   */
  status_inferred?: boolean;
  /** Error message if state detection failed (tweak still usable but with unknown state) */
  error?: string;
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
  is_windows_server: boolean;
  /** System uptime in seconds */
  uptime_seconds: number;
  /** OS install date as ISO 8601 string */
  install_date: string | null;
}

// Inspection Types
export interface RegistryMismatch {
  hive: string;
  key: string;
  value_name: string;
  expected_value: unknown;
  actual_value: unknown;
  value_type?: string;
  description: string;
  is_match: boolean;
}

export interface ServiceMismatch {
  name: string;
  expected_startup: string;
  actual_startup?: string;
  description: string;
  is_match: boolean;
}

export interface SchedulerMismatch {
  task_path: string;
  task_name: string;
  expected_state: string;
  actual_state?: string;
  description: string;
  is_match: boolean;
}

export interface OptionInspection {
  option_index: number;
  label: string;
  is_current: boolean;
  is_pending: boolean;
  registry_results: RegistryMismatch[];
  service_results: ServiceMismatch[];
  scheduler_results: SchedulerMismatch[];
  all_match: boolean;
}

export interface TweakInspection {
  tweak_id: string;
  options: OptionInspection[];
  matched_option_index?: number;
}

/** Device/system information from Win32_ComputerSystem */
export interface DeviceInfo {
  /** System manufacturer (e.g., "Dell Inc.", "ASUS") */
  manufacturer: string;
  /** System model (e.g., "XPS 15 9520") */
  model: string;
  /** System type (e.g., "x64-based PC") */
  system_type: string;
  /** PC type: Desktop, Laptop, Workstation, etc. */
  pc_type: string;
}

/** CPU information */
export interface CpuInfo {
  /** CPU name (e.g., "Intel Core i7-12700K") */
  name: string;
  /** Number of physical cores */
  cores: number;
  /** Number of logical processors (threads) */
  threads: number;
  /** CPU architecture (e.g., "x64") */
  architecture: string;
  /** Maximum clock speed in MHz */
  max_clock_mhz: number;
}

/** GPU information */
export interface GpuInfo {
  /** GPU name (e.g., "NVIDIA GeForce RTX 3080") */
  name: string;
  /** GPU memory in GB */
  memory_gb: number;
  /** Driver version */
  driver_version: string;
  /** Video processor/chip name */
  processor: string;
  /** Current refresh rate in Hz */
  refresh_rate: number;
  /** Video mode description (resolution + color depth) */
  video_mode: string;
}

/** Disk drive information */
export interface DiskInfo {
  /** Disk model name */
  model: string;
  /** Size in GB */
  size_gb: number;
  /** Drive type (e.g., "SSD", "HDD") */
  drive_type: string;
  /** Interface type (e.g., "NVMe", "SATA") */
  interface_type: string;
  /** Disk health status (e.g., "Healthy", "Warning") */
  health_status: string | null;
}

/** Memory (RAM) information */
export interface MemoryInfo {
  /** Total physical memory in GB */
  total_gb: number;
  /** Memory speed in MHz */
  speed_mhz: number;
  /** Memory type (e.g., "DDR4", "DDR5") */
  memory_type: string;
  /** Number of memory sticks */
  slots_used: number;
}

/** Motherboard information */
export interface MotherboardInfo {
  /** Manufacturer (e.g., "ASUS", "MSI", "Gigabyte") */
  manufacturer: string;
  /** Product name/model */
  product: string;
  /** BIOS version */
  bios_version: string;
}

/** Network adapter information */
export interface NetworkInfo {
  name: string;
  mac_address: string;
  ip_address: string;
  dhcp_enabled: boolean;
}

/** Hardware information */
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
  /** Total storage across all disks in GB */
  total_storage_gb: number;
}

/** System information */
export interface SystemInfo {
  windows: WindowsInfo;
  computer_name: string;
  username: string;
  is_admin: boolean;
  hardware: HardwareInfo;
  device: DeviceInfo;
}

/** Result of applying a tweak */
export interface TweakResult {
  success: boolean;
  message: string;
  requires_reboot: boolean;
  /** List of [tweak_id, error_message] for failed operations in batch mode */
  failures?: [string, string][];
}

/** Batch apply result */
export interface BatchApplyResult {
  success: boolean;
  results: Record<string, TweakResult>;
  total_applied: number;
  total_failed: number;
}

/** Pending change for staged apply pattern */
export interface PendingChange {
  /** Tweak ID */
  tweakId: string;
  /** Option index to apply */
  optionIndex: number;
}

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

// ============================================
// App Settings & Export/Import Types
// ============================================

/** App settings stored in localStorage */
export interface AppSettings {
  /** Whether to automatically check for updates */
  autoCheckUpdates: boolean;
  /** Whether to automatically install updates when available */
  autoInstallUpdates: boolean;
  /** Interval in hours between update checks */
  checkUpdateInterval: number;
  /** Last time an update check was performed (ISO 8601) */
  lastUpdateCheck: string | null;
}

/** Tweak snapshot for export - captures current registry state */
export interface TweakSnapshot {
  tweakId: string;
  tweakName: string;
  isApplied: boolean;
  /** Current registry values at time of snapshot */
  registryValues: Record<string, unknown>;
  /** Timestamp of snapshot */
  snapshotTime: string;
}

/** Export data structure for settings/tweaks */
export interface ExportData {
  version: string;
  exportTime: string;
  appVersion: string;
  settings: AppSettings;
  tweakSnapshots: TweakSnapshot[];
}

/** Update information from the backend */
export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion?: string;
  releaseNotes?: string;
  downloadUrl?: string;
  publishedAt?: string;
  /** Asset file name for download */
  assetName?: string;
  /** Asset size in bytes */
  assetSize?: number;
}

/** Update check result */
export interface UpdateCheckResult {
  success: boolean;
  update?: UpdateInfo;
  error?: string;
}

// ============================================================================
// PERMISSION LEVEL HELPERS
// ============================================================================

/** Permission info for UI display */
export interface PermissionInfo {
  name: string;
  description: string;
  icon: string;
  /** Color class for styling (e.g., 'text-foreground-muted', 'text-accent') */
  colorClass: string;
}

/** Permission level metadata for UI */
export const PERMISSION_INFO: Record<Exclude<PermissionLevel, "none">, PermissionInfo> = {
  admin: {
    name: "Admin",
    description: "Requires Administrator privileges to apply",
    icon: "mdi:shield-account-outline",
    colorClass: "text-foreground-muted",
  },
  system: {
    name: "System",
    description: "Requires SYSTEM elevation for protected registry keys and services",
    icon: "mdi:shield-lock",
    colorClass: "text-accent",
  },
  ti: {
    name: "TrustedInstaller",
    description: "Requires TrustedInstaller elevation for highly protected resources",
    icon: "mdi:shield-key",
    colorClass: "text-warning",
  },
};

/**
 * Get the highest permission level from a tweak definition.
 * Permission hierarchy: ti > system > admin > none
 *
 * @param tweak - Object with requires_admin, requires_system, requires_ti flags
 * @returns The highest permission level required
 */
export function getHighestPermission(tweak: {
  requires_admin: boolean;
  requires_system: boolean;
  requires_ti: boolean;
}): PermissionLevel {
  if (tweak.requires_ti) return "ti";
  if (tweak.requires_system) return "system";
  if (tweak.requires_admin) return "admin";
  return "none";
}
