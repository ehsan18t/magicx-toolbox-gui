type DateInput = string | number | Date;

const KIB = 1024;
const MIB = KIB * KIB;

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${n} ${n === 1 ? one : many}`;
}

function toDate(value: DateInput): Date | null {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

/** Unparseable input comes back as is, never as "Invalid Date". */
export function formatDate(value: DateInput, month: "long" | "short" | "numeric" = "numeric"): string {
  const date = toDate(value);
  return date ? date.toLocaleDateString(undefined, { year: "numeric", month, day: "numeric" }) : String(value);
}

export function formatDateTime(value: DateInput, dateStyle?: "long"): string {
  const date = toDate(value);
  if (!date) return String(value);
  return dateStyle ? date.toLocaleString(undefined, { dateStyle, timeStyle: "short" }) : date.toLocaleString();
}

export function formatBytes(bytes: number): string {
  return bytes >= MIB ? `${(bytes / MIB).toFixed(1)} MB` : `${Math.ceil(bytes / KIB)} KB`;
}
