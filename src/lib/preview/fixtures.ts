import type { LogLine, LogSettings } from "$lib/types";
import type {
  AppPresence,
  Attention,
  Availability,
  EntrySummary,
  HeldInfo,
  InstallRoute,
  SystemInfo,
  TweakStateView,
  UnavailableOpt,
} from "$lib/types";

export const APP_VERSION = "3.0.0";

export interface Scenario {
  state: TweakStateView;
  has_history?: boolean;
  attention?: Attention;
  residues?: string[];
  held_shared?: HeldInfo[];
  unavailable?: UnavailableOpt[];
  entries?: EntrySummary[];
}

const ASR_HELD: HeldInfo = {
  shared: "defender_asr_policy_enabled",
  holders: ["asr_block_office_script_vectors", "asr_block_lsass_theft"],
};

const entry = (seq: number, daysAgo: number, validity: EntrySummary["validity"] = "Valid"): EntrySummary => ({
  seq,
  validity,
  timestamp: new Date(Date.now() - daysAgo * 86_400_000).toISOString(),
  captured: null,
});

export const scenarios: Record<string, Scenario> = {
  disable_vbs_hvci: {
    state: { state: "active", option: "Disabled" },
    has_history: true,
    attention: {
      reason: "restore_failed",
      items: [
        {
          effect: "enable_vbs",
          kind: "drive",
          class: "access_denied",
          entries: [3],
          message:
            "Writing EnableVirtualizationBasedSecurity under HKLM\\SYSTEM\\CurrentControlSet\\Control\\DeviceGuard was refused (access denied)",
        },
        {
          effect: "hvci_enabled",
          kind: "verify",
          class: "invalid_data",
          message: "Memory integrity read back 0 after the restore wrote 1; Windows may be enforcing it from firmware",
        },
        { effect: null, kind: "store", message: "The snapshot entry could not be released after the partial restore" },
      ],
    },
    residues: ["hvci_enabled"],
    held_shared: [{ shared: "device_guard_platform", holders: ["enable_credential_guard", "disable_vbs_hvci"] }],
    unavailable: [{ label: "Memory integrity enabled", reason: "not applicable on this Windows build" }],
    entries: [entry(3, 0.2), entry(2, 6, { Invalid: "WrongMachine" }), entry(1, 40, { Invalid: "Corrupt" })],
  },
  enable_credential_guard: {
    state: { state: "active", option: "Off" },
    has_history: true,
    attention: {
      reason: "apply_failed",
      items: [
        {
          effect: "lsa_cfg_flags",
          kind: "verify",
          class: "failed",
          message: "LsaCfgFlags read back 0 after writing 1",
        },
      ],
    },
  },
  disable_search_indexing: {
    state: { state: "system_default" },
    has_history: true,
    attention: {
      reason: "crash_residue",
      items: [
        {
          effect: "wsearch",
          kind: "crash_residue",
          entries: [1],
          message: "WSearch start type change was never recorded as finished",
        },
      ],
    },
  },
  disable_diagnostic_data: {
    state: { state: "active", option: "Required only" },
    has_history: true,
    attention: {
      reason: "outcome_unrecorded",
      items: [
        {
          effect: null,
          kind: "unrecorded",
          message: "The apply verified, but its outcome could not be written to the snapshot store",
        },
      ],
    },
  },
  disable_widgets: {
    state: { state: "active", option: "Off" },
    has_history: true,
    attention: {
      reason: "record_unreadable",
      items: [
        {
          effect: null,
          kind: "store",
          class: "invalid_data",
          message: "disable_widgets.attention.json is not valid JSON (line 1, column 214)",
        },
      ],
    },
  },
  asr_block_office_script_vectors: {
    state: { state: "active", option: "Audit only" },
    has_history: true,
    held_shared: [ASR_HELD],
  },
  asr_block_lsass_theft: { state: { state: "active", option: "Block" }, has_history: true, held_shared: [ASR_HELD] },
  remove_smbv1: { state: { state: "active", option: "Removed" }, has_history: true, residues: ["smb1_feature"] },
  defender_cloud_protection: {
    state: { state: "active", option: "High, prompt before sending samples" },
    unavailable: [
      {
        label: "High, send safe samples",
        reason:
          "requires effect 'block_at_first_seen' to have a real value, but its resource is not present on this machine",
      },
    ],
  },
  disable_explorer_cloud_recommendations: {
    state: {
      state: "unavailable",
      reason: "'disable_explorer_cloud_recommendations' has no applicable effects on this Windows build",
    },
  },
  disable_compat_appraiser: {
    state: {
      state: "unknown",
      reasons: [
        { effect: "task_appraiser", cause: "AccessDenied", needs_elevation: true },
        { effect: "task_marebackup", cause: "AccessDenied", needs_elevation: true },
      ],
    },
  },
  memory_prefetch_mode: {
    state: { state: "unknown", reasons: [{ effect: "sysmain", cause: "Malformed", needs_elevation: false }] },
  },
  alt_tab_hide_browser_tabs: {
    state: { state: "active", option: "Windows and 20 most recent tabs" },
    has_history: true,
  },
  windows_update_mode: {
    state: { state: "active", option: "Notify only: nothing downloads unprompted" },
    has_history: true,
  },
  update_feature_control: { state: { state: "system_default" } },
  block_update_pipeline: { state: { state: "system_default" } },
};

/** Never emitted, so the card stays in its loading state. */
export const NEVER_LOADS = "disable_edge_startup_boost";

export const PENDING_REBOOT = [
  "disable_vbs_hvci",
  "remove_smbv1",
  "block_vulnerable_drivers",
  "disable_consumer_features",
];

const SID_MISMATCH =
  "Another account elevated this app, so per-user tweaks stay off until you restart it under your own account.";
const SID_UNKNOWN =
  "This app could not confirm which account owns this session, so per-user tweaks stay off rather than risk changing the wrong account's settings.";

/** Admin-mode overrides; a non-admin preview computes needs_elevation instead. */
export const adminAvailability: Record<string, Availability> = {
  lock_on_inactivity: { state: "sid_mismatch", reason: SID_MISMATCH },
  disable_language_list_access: { state: "sid_unknown", reason: SID_UNKNOWN },
  block_update_pipeline: {
    state: "elevation_path_unavailable",
    reason:
      "The Windows Modules Installer (TrustedInstaller) service is disabled on this PC, so tweaks that need it cannot run. Set its startup type to Manual to enable them.",
  },
};

export const appAdminAvailability: Record<string, Availability> = {
  onedrive: {
    state: "sid_mismatch",
    reason:
      "Another account elevated this app, so it would change that account's apps. Restart it under your own account.",
  },
};

export function appPresence(admin: boolean): Record<string, { presence: AppPresence; install_route: InstallRoute }> {
  const installed = { presence: { state: "installed", provisioned_only: false }, install_route: "winget" } as const;
  return {
    teams_consumer: installed,
    clipchamp: { presence: { state: "installed", provisioned_only: true }, install_route: "winget" },
    quick_assist: installed,
    bing_news: { presence: { state: "absent" }, install_route: "winget" },
    bing_weather: installed,
    solitaire: { presence: { state: "absent" }, install_route: "store_page" },
    get_help: installed,
    getstarted_tips: { presence: { state: "absent" }, install_route: "none" },
    feedback_hub: installed,
    phone_link: {
      presence: admin
        ? { state: "unknown", reason: "The installed app list could not be read.", needs_elevation: false }
        : {
            state: "unknown",
            reason: "Without administrator rights only this account's apps can be checked.",
            needs_elevation: true,
          },
      install_route: "winget",
    },
    outlook_new: installed,
    xbox_game_bar: { presence: { state: "installed", provisioned_only: true }, install_route: "store_page" },
    onedrive: installed,
  };
}

export function systemInfo(admin: boolean): SystemInfo {
  return {
    windows: {
      product_name: "Windows 11 Pro for Workstations",
      display_version: "25H2",
      build_number: "26200.6584",
      is_windows_11: true,
      version_string: "11",
      is_windows_server: false,
      uptime_seconds: 3 * 86_400 + 7 * 3_600 + 42 * 60,
      install_date: "2025-11-14T09:21:07Z",
    },
    computer_name: "WORKSTATION-7F3K",
    username: "PreviewUser",
    is_admin: admin,
    hardware: {
      cpu: {
        name: "AMD Ryzen Threadripper PRO 7995WX 96-Cores",
        cores: 96,
        threads: 192,
        architecture: "x64",
        max_clock_mhz: 5100,
      },
      gpu: [
        {
          name: "NVIDIA RTX 6000 Ada Generation",
          memory_gb: 48,
          driver_version: "32.0.15.6614",
          processor: "NVIDIA RTX 6000 Ada Generation",
          refresh_rate: 144,
          video_mode: "3840 x 2160 x 4294967296 colors",
        },
        {
          name: "Intel(R) Iris(R) Xe Graphics Family (Microsoft WDDM 3.1 Hybrid Adapter)",
          memory_gb: 2,
          driver_version: "31.0.101.5594",
          processor: "Intel(R) Iris(R) Xe Graphics Family",
          refresh_rate: 60,
          video_mode: "2560 x 1440 x 4294967296 colors",
        },
      ],
      monitors: [
        { name: "Dell UltraSharp U3224KB 6K Thunderbolt Hub Monitor", resolution: "6144 x 3456", refresh_rate: 60 },
        { name: "Generic PnP Monitor", resolution: "2560 x 1440", refresh_rate: 144 },
      ],
      memory: { total_gb: 256, speed_mhz: 5600, memory_type: "DDR5", slots_used: 8 },
      motherboard: { manufacturer: "ASUSTeK COMPUTER INC.", product: "Pro WS WRX90E-SAGE SE", bios_version: "1203" },
      disks: [
        {
          model: "Samsung SSD 990 PRO with Heatsink 4TB",
          size_gb: 3726,
          drive_type: "SSD",
          interface_type: "NVMe",
          health_status: "Healthy",
        },
        {
          model: "WDC WD201KFGX-68BKJN0",
          size_gb: 18626,
          drive_type: "HDD",
          interface_type: "SATA",
          health_status: "Warning",
        },
      ],
      network: [
        {
          name: "Intel(R) Ethernet Controller X710 for 10GbE SFP+ #2",
          mac_address: "3C:FD:FE:A1:22:7B",
          ip_address: "192.168.1.42",
          dhcp_enabled: true,
        },
        { name: "Intel(R) Wi-Fi 7 BE200 320MHz", mac_address: "A4:42:3B:19:E0:5C", ip_address: "", dhcp_enabled: true },
      ],
      total_storage_gb: 22352,
    },
    device: {
      manufacturer: "Micro-Star International Co., Ltd.",
      model: "MS-7E12 Creator WRX90 Workstation",
      system_type: "x64-based PC",
      pc_type: "Workstation",
    },
  };
}

export const logSettings: LogSettings = {
  persist: true,
  detailed: false,
  folder: "C:\\Tools\\MagicX Toolbox\\logs",
  writing: true,
  error: null,
  files: 3,
  bytes: 482_113,
};

const LOG_SEED: Omit<LogLine, "seq" | "ts">[] = [
  {
    level: "info",
    source: "app",
    target: "magicx_toolbox",
    msg: "MagicX Toolbox 3.0.0 starting (Windows 11 build 26200.6584)",
  },
  {
    level: "info",
    source: "app",
    target: "magicx_toolbox::commands::tweaks",
    msg: "get_tweaks: compiled corpus for the UI",
  },
  {
    level: "debug",
    source: "app",
    target: "magicx_toolbox::tweaks::engine::detect",
    msg: "scan started for 49 tweaks",
  },
  {
    level: "warn",
    source: "app",
    target: "magicx_toolbox::tweaks::engine::detect",
    msg: "disable_compat_appraiser: task \\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser could not be read (access denied)",
  },
  {
    level: "error",
    source: "app",
    target: "magicx_toolbox::tweaks::engine::revert",
    msg: "disable_vbs_hvci: restore did not fully complete; Needs Attention recorded",
  },
  { level: "info", source: "helper", target: "broker", msg: "TrustedInstaller broker unavailable: service disabled" },
  { level: "info", source: "ui", target: "frontend", msg: "Preview mode: IPC answered by fixtures" },
];

export function seedLogLines(): LogLine[] {
  const now = Date.now();
  return LOG_SEED.map((l, i) => ({
    ...l,
    seq: i + 1,
    ts: new Date(now - (LOG_SEED.length - i) * 1_500).toISOString(),
  }));
}
