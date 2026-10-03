const KIB = 1024;
const MIB = KIB * KIB;

export function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** Empty for negative or non-finite input. */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "";
  const kib = Math.ceil(bytes / KIB);
  return kib >= KIB ? `${(bytes / MIB).toFixed(1)} MB` : `${kib} KB`;
}
