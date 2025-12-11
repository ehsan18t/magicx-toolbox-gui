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
  /** Requires SYSTEM elevation for protected registry keys */
  requires_system: boolean;
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
  /** System uptime in seconds */
  uptime_seconds: number;
  /** OS install date as ISO 8601 string */
  install_date: string | null;
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

/** Hardware information */
export interface HardwareInfo {
  cpu: CpuInfo;
  gpu: GpuInfo[];
  memory: MemoryInfo;
  motherboard: MotherboardInfo;
  disks: DiskInfo[];
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
