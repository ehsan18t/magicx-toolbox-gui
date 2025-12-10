// Type definitions matching Rust models

/** Risk level for tweaks */
export type RiskLevel = "low" | "medium" | "high" | "critical";

/** Categories for organizing tweaks */
export type TweakCategory = "privacy" | "performance" | "ui" | "security" | "services" | "gaming";

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

/** Single registry change operation */
export interface RegistryChange {
  hive: RegistryHive;
  key: string;
  value_name: string;
  value_type: RegistryValueType;
  enable_value: unknown;
  disable_value?: unknown;
}

/** A complete tweak definition loaded from YAML */
export interface TweakDefinition {
  id: string;
  name: string;
  description: string;
  category: TweakCategory;
  risk_level: RiskLevel;
  requires_reboot: boolean;
  requires_admin: boolean;
  /** Map of Windows version ("10" or "11") to registry changes */
  registry_changes: Record<string, RegistryChange[]>;
  /** Additional info/documentation */
  info?: string;
}

/** Status of a tweak in the system */
export interface TweakStatus {
  tweak_id: string;
  is_applied: boolean;
  last_applied?: string; // ISO 8601 timestamp
  has_backup: boolean;
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

/** Category display info */
export interface CategoryInfo {
  id: TweakCategory;
  name: string;
  description: string;
  icon: string;
}

/** All category info */
export const CATEGORY_INFO: Record<TweakCategory, CategoryInfo> = {
  privacy: {
    id: "privacy",
    name: "Privacy",
    description: "Reduce telemetry, tracking, and data collection",
    icon: "🔒",
  },
  performance: {
    id: "performance",
    name: "Performance",
    description: "Optimize system speed and responsiveness",
    icon: "⚡",
  },
  ui: {
    id: "ui",
    name: "UI/UX",
    description: "Customize Windows appearance and behavior",
    icon: "🎨",
  },
  security: {
    id: "security",
    name: "Security",
    description: "Improve Windows security settings",
    icon: "🛡️",
  },
  services: {
    id: "services",
    name: "Services",
    description: "Manage unnecessary Windows services",
    icon: "⚙️",
  },
  gaming: {
    id: "gaming",
    name: "Gaming",
    description: "Optimize Windows for gaming performance",
    icon: "🎮",
  },
};

/** Risk level display info */
export const RISK_INFO: Record<RiskLevel, { name: string; color: string; description: string }> = {
  low: {
    name: "Low",
    color: "text-green-500",
    description: "Safe to apply/revert without issues",
  },
  medium: {
    name: "Medium",
    color: "text-yellow-500",
    description: "May require restart or have minor side effects",
  },
  high: {
    name: "High",
    color: "text-orange-500",
    description: "Could significantly impact system",
  },
  critical: {
    name: "Critical",
    color: "text-red-500",
    description: "Could break Windows, use with caution",
  },
};
