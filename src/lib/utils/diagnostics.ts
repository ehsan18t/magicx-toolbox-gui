import { appInfoStore } from "$lib/stores/appInfo.svelte";
import { systemStore } from "$lib/stores/system.svelte";

export interface DiagnosticsFact {
  label: string;
  value: string;
}

export const versionLabel = (): string =>
  appInfoStore.version ? `Version ${appInfoStore.version}` : "Version unknown";

/** Null until loaded: system info and the engine version load asynchronously. */
function allFacts(): { label: string; value: string | null }[] {
  const info = systemStore.info;
  return [
    { label: "Windows", value: info ? `${info.windows.product_name} ${info.windows.display_version}` : null },
    { label: "Build", value: info?.windows.build_number ?? null },
    { label: "Running as", value: info ? (info.is_admin ? "Administrator" : "Standard user") : null },
    { label: "Engine", value: appInfoStore.tauriVersion ? `Tauri ${appInfoStore.tauriVersion}` : null },
  ];
}

/** Only the facts known so far. */
export function diagnosticsFacts(): DiagnosticsFact[] {
  return allFacts().filter((f): f is DiagnosticsFact => f.value !== null);
}

/** The lines a bug report starts with; a fact not loaded yet reads "unknown", so every report has every line. */
export function diagnosticsHeader(): string {
  const facts = allFacts().map((f) => `${f.label}: ${f.value ?? "unknown"}`);
  return [appInfoStore.name, versionLabel(), ...facts].join("\n");
}
