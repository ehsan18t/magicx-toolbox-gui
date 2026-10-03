import { closeWindowOrHint } from "$lib/api/system";
import { getPreviousVersionHint } from "$lib/api/update";
import { errorMessage } from "$lib/utils/error";
import { installErrorForwarding, logError } from "$lib/utils/logger";
import { mount } from "svelte";

/** Plain DOM over the loader: Svelte or the app's CSS may be what failed. */
function showStartupError(error: unknown): void {
  const host = document.getElementById("initial-loader") ?? document.body.appendChild(document.createElement("div"));
  host.id = "initial-loader";
  host.classList.remove("fade-out");

  const panel = document.createElement("div");
  panel.className = "startup-error";
  panel.setAttribute("role", "alert");
  const title = document.createElement("h1");
  title.textContent = "MagicX Toolbox could not start";
  const detail = document.createElement("p");
  detail.textContent = errorMessage(error);
  const hint = document.createElement("p");
  hint.textContent = "The session log has the details.";
  const close = document.createElement("button");
  close.type = "button";
  close.textContent = "Close";
  close.addEventListener("click", () => {
    void closeWindowOrHint().then((failed) => {
      if (failed) hint.textContent = failed;
    });
  });

  panel.append(title, detail, hint, close);
  host.replaceChildren(panel);
  close.focus();

  getPreviousVersionHint()
    .then((text) => {
      if (!text) return;
      const previous = document.createElement("p");
      previous.textContent = text;
      hint.after(previous);
    })
    .catch((error) => logError("Failed to look for the previous version", error));
}

// Dev-only browser preview with fixture IPC: `?preview` (admin) or `?preview&user`. DEV-gated so builds drop it.
async function start(): Promise<void> {
  installErrorForwarding();
  try {
    const params = new URLSearchParams(location.search);
    if (import.meta.env.DEV && params.has("preview")) {
      const { installPreview } = await import("$lib/preview");
      installPreview(!params.has("user"));
    }
    // Imported here so a module that throws while loading is caught too.
    const { default: App } = await import("./App.svelte");
    mount(App, { target: document.getElementById("app")! });
  } catch (error) {
    logError("The interface failed to start", error);
    showStartupError(error);
    return;
  }
}

void start();
