import type { ColumnTone, IconName } from "$lib/design";
import type { EffectAgreement, RegistryChange, RegistryValueType, TweakEffectOption } from "$lib/types";
import { capitalize } from "$lib/utils/format";

export type ChangeKind = "registry" | "service" | "task" | "hosts" | "firewall";

export const KIND_META: Record<ChangeKind, { label: string; icon: IconName }> = {
  registry: { label: "Registry", icon: "mdi:database" },
  service: { label: "Services", icon: "mdi:server" },
  task: { label: "Scheduled tasks", icon: "mdi:calendar" },
  hosts: { label: "Hosts file", icon: "mdi:file-document-outline" },
  firewall: { label: "Firewall", icon: "mdi:shield-outline" },
};

export function optionTone(label: string, activeOption: string | null, pendingLabel?: string): ColumnTone | null {
  if (label === activeOption) return "current";
  if (label === pendingLabel) return "pending";
  return null;
}

export interface MatrixCell {
  text: string;
  /** Removes rather than sets something: shown subdued. */
  removal?: boolean;
  /** Limits the change to some Windows versions, e.g. "Win 11". */
  note?: string;
}

export interface MatrixRow {
  kind: ChangeKind;
  title: string;
  /** The engine's name for this setting, as `EffectAgreement.name` carries it. */
  name: string;
  /** Where the setting lives: a registry key path, a task folder, or the kind of item. */
  location: string;
  type?: RegistryValueType;
  /** One per option, in option order; null leaves the setting untouched. */
  cells: (MatrixCell | null)[];
  /** The live value at System Default, when the engine reported one. */
  now: MatrixCell | null;
}

const lastSegment = (path: string) => path.split("\\").filter(Boolean).at(-1) ?? path;

const humanize = (s: string) => capitalize(s.replaceAll("_", " "));

function registryCell(c: RegistryChange): MatrixCell {
  const scope = c.windows_versions?.length ? { note: `Win ${c.windows_versions.join(", ")}` } : {};
  if (c.action === "delete_value") return { text: "Not set", removal: true, ...scope };
  if (c.action === "delete_key") return { text: "Key removed", removal: true, ...scope };
  if (c.action === "create_key") return { text: "Key created", ...scope };
  const v = c.value;
  const text =
    v === null
      ? "(empty)"
      : typeof v === "number"
        ? `${v} (0x${v.toString(16).toUpperCase()})`
        : typeof v === "string"
          ? v === ""
            ? '""'
            : v
          : JSON.stringify(v);
  return { text, ...scope };
}

interface MatrixEntry {
  key: string;
  row: Omit<MatrixRow, "cells" | "now">;
  cell: MatrixCell;
}

function matrixEntries(o: TweakEffectOption): MatrixEntry[] {
  return [
    ...o.registry_changes.map((c) => {
      const isKey = c.action === "delete_key" || c.action === "create_key";
      return {
        key: `reg:${c.hive}\\${c.key}\\${isKey ? "" : c.value_name}`.toLowerCase(),
        row: {
          kind: "registry" as const,
          title: isKey ? "(key)" : c.value_name || "(Default)",
          name: isKey ? lastSegment(c.key) : c.value_name,
          location: `${c.hive}\\${c.key}`,
          ...(!isKey && c.value_type !== null && { type: c.value_type }),
        },
        cell: registryCell(c),
      };
    }),
    ...o.service_changes.map((c) => ({
      key: `svc:${c.name}`.toLowerCase(),
      row: { kind: "service" as const, title: c.name, name: c.name, location: "Service startup" },
      cell: { text: humanize(c.startup) },
    })),
    ...o.scheduler_changes.map((c) => {
      const cut = c.task_path.lastIndexOf("\\");
      return {
        key: `task:${c.task_path}`.toLowerCase(),
        row: {
          kind: "task" as const,
          title: c.task_path.slice(cut + 1) || c.task_path,
          name: lastSegment(c.task_path),
          location: cut > 0 ? c.task_path.slice(0, cut) : "Task Scheduler",
        },
        cell: { text: c.action === "enable" ? "Enabled" : "Disabled" },
      };
    }),
    ...o.hosts_changes.map((c) => ({
      key: `hosts:${c.domain}`.toLowerCase(),
      row: { kind: "hosts" as const, title: c.domain, name: c.domain, location: "Hosts file" },
      cell: c.action === "add" ? { text: `Mapped to ${c.ip}` } : { text: "Not listed", removal: true },
    })),
    ...o.firewall_changes.map((c) => ({
      key: `fw:${c.name}`.toLowerCase(),
      row: { kind: "firewall" as const, title: c.name, name: c.name, location: "Firewall rule" },
      cell:
        c.operation === "delete"
          ? { text: "Removed", removal: true }
          : { text: `${humanize(c.action)} ${c.direction}` },
    })),
  ];
}

const KIND_ORDER: ChangeKind[] = ["registry", "service", "task", "hosts", "firewall"];

/** One row per setting any option touches, so options compare side by side. */
export function buildMatrix(options: TweakEffectOption[], observed: TweakEffectOption | null): MatrixRow[] {
  const rows = new Map<string, MatrixRow>();
  const rowFor = (e: MatrixEntry) => {
    let row = rows.get(e.key);
    if (!row) {
      row = { ...e.row, cells: options.map(() => null), now: null };
      rows.set(e.key, row);
    }
    return row;
  };
  options.forEach((o, i) => {
    for (const e of matrixEntries(o)) rowFor(e).cells[i] = e.cell;
  });
  if (observed) {
    for (const e of matrixEntries(observed)) rowFor(e).now = e.cell;
  }
  return [...rows.values()].sort((a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind));
}

/**
 * Which options would leave this row at the live value. The engine's agreement wins: it compares real
 * values with Windows-version scoping. Text comparison is only the fallback for an ambiguous name.
 */
export function optionsMatchingNow(row: MatrixRow, labels: string[], agreement: EffectAgreement[]): string[] {
  if (!row.now) return [];
  // The engine names a registry value's sub-field `name [field]`, so both forms belong to this row.
  const own = agreement.filter((a) => a.name === row.name || a.name.startsWith(`${row.name} [`));
  if (own.length === 1) return own[0].wanted_by;
  return labels.filter((_, i) => row.cells[i]?.text === row.now?.text);
}
