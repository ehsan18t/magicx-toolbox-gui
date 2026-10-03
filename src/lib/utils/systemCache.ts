import type { CachedSystemInfo, LiveSystemInfo, MachineHardware, SystemInfo } from "$lib/types";

/** Stands in for the live fields when only the cached hardware could be read. */
export const UNKNOWN_LIVE: LiveSystemInfo = {
  windows: {
    version_string: "",
    display_version: "",
    build_number: "",
    product_name: "Windows",
    uptime_seconds: 0,
    is_windows_11: false,
    install_date: null,
  },
  computer_name: "",
  username: "",
  is_admin: false,
};

const isObject = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null;

/** The shape systemInfoRows destructures: a cache from an older build would otherwise crash the render. */
export function parseSystemCache(stored: unknown): CachedSystemInfo | null | undefined {
  if (stored === null) return null;
  if (!isObject(stored) || !isObject(stored.hardware) || !isObject(stored.device)) return undefined;
  const { cpu, memory, motherboard, gpu, monitors, disks, network } = stored.hardware;
  const valid =
    typeof stored.cachedAt === "string" &&
    [cpu, memory, motherboard].every(isObject) &&
    [gpu, monitors, disks, network].every(Array.isArray);
  return valid ? ({ ...stored, partial: stored.partial === true } as CachedSystemInfo) : undefined;
}

/** A partial read is cached only while nothing complete is: it never overwrites a complete one. */
export const replacesCache = (cached: CachedSystemInfo | null, machine: MachineHardware): boolean =>
  !machine.partial || !cached || cached.partial;

export const composeSystemInfo = (
  live: LiveSystemInfo,
  { hardware, device, partial }: MachineHardware,
): SystemInfo => ({
  ...live,
  hardware,
  device,
  partial,
});

/** What the card paints before the hardware read: the cache under the live fields (placeholders if unread), or null for a skeleton. */
export function firstPaint(live: LiveSystemInfo | null, cached: CachedSystemInfo | null): SystemInfo | null {
  return cached ? composeSystemInfo(live ?? UNKNOWN_LIVE, cached) : null;
}
