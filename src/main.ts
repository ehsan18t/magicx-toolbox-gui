import { mount } from "svelte";
import App from "./App.svelte";

// Dev-only browser preview with fixture IPC: `?preview` (admin) or `?preview&user`. DEV-gated so builds drop it.
async function start(): Promise<void> {
  const params = new URLSearchParams(location.search);
  if (import.meta.env.DEV && params.has("preview")) {
    const { installPreview } = await import("$lib/preview");
    installPreview(!params.has("user"));
  }
  mount(App, { target: document.getElementById("app")! });
}

void start();
