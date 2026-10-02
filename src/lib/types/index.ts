// Mirrors the serde shapes emitted by src-tauri/src/commands/*.rs unless noted.

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

/**
 * `sid_mismatch` positively identified another account; `sid_unknown` could not read a SID. Only the first
 * may say another account is involved. `elevation_path_unavailable` (TrustedInstaller disabled or absent)
 * is not fixed by restarting as administrator, so it never says to.
 */
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

/** Kept per tweak, not per snapshot entry, so releasing an entry cannot drop it (ADR-0001/0002). */
export interface Attention {
  reason: "apply_failed" | "restore_failed" | "crash_residue" | "outcome_unrecorded" | "record_unreadable";
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

/**
 * `sid_mismatch` covers both a confirmed other account and an unresolvable one; only the per-tweak
 * `Availability` tells them apart. Never render it as "another account elevated the app".
 */
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
  /** Iconify name, e.g. "mdi:shield-lock". */
  icon: string;
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
  /** The engine's has_history. */
  hasSnapshot: boolean;
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
export interface CachedSystemInfo {
  hardware: HardwareInfo;
  device: DeviceInfo;
  computer_name: string;
  cachedAt: string;
}

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
