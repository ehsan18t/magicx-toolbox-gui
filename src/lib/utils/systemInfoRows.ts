import type { IconName } from "$lib/components/shared";
import type { TextTone } from "$lib/components/ui/tone";
import type { SystemInfo } from "$lib/types";

const MHZ_PER_GHZ = 1000;
const GB_PER_TB = 1000;
const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY = 24 * SECONDS_PER_HOUR;
const SEP = " · ";

export interface HardwareRow {
  icon: IconName;
  label: string;
  value: string;
  detail: string;
  status?: { text: string; tone: TextTone };
}

const formatClock = (mhz: number) => (mhz >= MHZ_PER_GHZ ? `${(mhz / MHZ_PER_GHZ).toFixed(1)} GHz` : `${mhz} MHz`);

const formatStorage = (gb: number) => (gb >= GB_PER_TB ? `${(gb / GB_PER_TB).toFixed(1)} TB` : `${gb.toFixed(0)} GB`);

function formatUptime(seconds: number): string {
  if (!seconds || seconds <= 0) return "Unknown";
  const days = Math.floor(seconds / SECONDS_PER_DAY);
  const hours = Math.floor((seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR);
  const minutes = Math.floor((seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

const details = (...parts: (string | false | null | undefined)[]) => parts.filter(Boolean).join(SEP);

const numbered = (label: string, i: number, count: number) => (count > 1 ? `${label} ${i + 1}` : label);

/** `summary` describes the PC itself; `devices` lists each GPU, display, disk and network adapter. */
export function systemInfoRows(info: SystemInfo): { summary: HardwareRow[]; devices: HardwareRow[] } {
  const { cpu, gpu, monitors, memory, motherboard, disks, network } = info.hardware;
  const summary: HardwareRow[] = [
    {
      icon: "mdi:microsoft-windows",
      label: "Windows",
      value: info.windows.product_name,
      detail: `${info.windows.display_version}${SEP}build ${info.windows.build_number}`,
    },
    {
      icon: info.device.pc_type === "Laptop" ? "mdi:laptop" : "mdi:desktop-tower-monitor",
      label: "Device",
      value: info.device.model || info.computer_name,
      detail: info.device.manufacturer,
    },
    {
      icon: "mdi:timer-outline",
      label: "Uptime",
      value: formatUptime(info.windows.uptime_seconds),
      detail: "since last restart",
    },
    {
      icon: "mdi:cpu-64-bit",
      label: "Processor",
      value: cpu.name,
      detail: `${cpu.cores} cores, ${cpu.threads} threads${SEP}up to ${formatClock(cpu.max_clock_mhz)}`,
    },
    {
      icon: "ri:ram-line",
      label: "Memory",
      value: `${memory.total_gb} GB ${memory.memory_type}`.trim(),
      detail: details(
        memory.speed_mhz > 0 && `${memory.speed_mhz} MHz`,
        memory.slots_used > 0 && `${memory.slots_used} modules`,
      ),
    },
    {
      icon: "bi:motherboard",
      label: "Motherboard",
      value: motherboard.product,
      detail: details(motherboard.manufacturer, motherboard.bios_version && `BIOS ${motherboard.bios_version}`),
    },
  ];
  const devices: HardwareRow[] = [
    ...gpu.map((g, i): HardwareRow => ({
      icon: "mdi:expansion-card",
      label: numbered("Graphics", i, gpu.length),
      value: g.name,
      detail: details(
        g.memory_gb > 0 ? `${g.memory_gb} GB` : "Shared memory",
        g.driver_version && `Driver ${g.driver_version}`,
        monitors.length === 0 && g.refresh_rate > 0 && `${g.refresh_rate} Hz`,
      ),
    })),
    ...monitors.map((m, i): HardwareRow => ({
      icon: "mdi:monitor",
      label: numbered("Display", i, monitors.length),
      value: m.name,
      detail: details(m.resolution, m.refresh_rate > 0 && `${m.refresh_rate} Hz`),
    })),
    ...disks.map((d, i): HardwareRow => ({
      icon: d.drive_type === "SSD" ? "mdi:harddisk" : "mdi:harddisk-plus",
      label: numbered("Storage", i, disks.length),
      value: d.model,
      detail: details(formatStorage(d.size_gb), d.drive_type, d.interface_type !== "Unknown" && d.interface_type),
      status: d.health_status
        ? { text: d.health_status, tone: d.health_status === "Healthy" ? "success" : "warning" }
        : undefined,
    })),
    ...network.map((n, i): HardwareRow => ({
      icon: "mdi:ethernet",
      label: numbered("Network", i, network.length),
      value: n.name,
      detail: details(n.ip_address || "Not connected", n.mac_address),
    })),
  ];
  return { summary, devices };
}
