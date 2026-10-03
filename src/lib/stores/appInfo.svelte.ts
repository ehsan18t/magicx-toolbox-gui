import { APP_CONFIG } from "$lib/config/app";
import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";

let name = $state<string>(APP_CONFIG.appName);
let version = $state("");
let tauriVersion = $state("");
let loading: Promise<void> | null = null;

/** Versions read "" until `load()` resolves, or when their read failed. */
export const appInfoStore = {
  get name() {
    return name;
  },
  get version() {
    return version;
  },
  get tauriVersion() {
    return tauriVersion;
  },

  /** Reads once; later calls share the first read. */
  load(): Promise<void> {
    loading ??= Promise.allSettled([getName(), getVersion(), getTauriVersion()]).then(([n, v, t]) => {
      if (n.status === "fulfilled") name = n.value;
      if (v.status === "fulfilled") version = v.value;
      if (t.status === "fulfilled") tauriVersion = t.value;
    });
    return loading;
  },
};
