import { appInfoStore } from "$lib/stores/appInfo.svelte";
import { systemStore } from "$lib/stores/system.svelte";

export interface DiagnosticsFact {
  label: string;
  value: string;
}

export const versionLabel = (): string =>
  appInfoStore.version ? `Version ${appInfoStore.version}` : "Version unknown";

/** Only the facts known so far: system info and the engine version load asynchronously. */
export function diagnosticsFacts(): DiagnosticsFact[] {
  const info = systemStore.info;
  const facts: { label: string; value: string | null }[] = [
    { label: "Windows", value: info ? `${info.windows.product_name} ${info.windows.display_version}` : null },
    { label: "Build", value: info?.windows.build_number ?? null },
    { label: "Running as", value: info ? (info.is_admin ? "Administrator" : "Standard user") : null },
    { label: "Engine", value: appInfoStore.tauriVersion ? `Tauri ${appInfoStore.tauriVersion}` : null },
  ];
  return facts.filter((f): f is DiagnosticsFact => f.value !== null);
}

/** The lines a bug report starts with. */
export function diagnosticsHeader(): string {
  return [appInfoStore.name, versionLabel(), ...diagnosticsFacts().map((f) => `${f.label}: ${f.value}`)].join("\n");
}
