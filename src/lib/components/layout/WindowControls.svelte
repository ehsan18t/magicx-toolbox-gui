<script lang="ts">
  import { logError } from "$lib/utils/logger";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import WindowControlButton from "./WindowControlButton.svelte";

  const appWindow = getCurrentWindow();
  let isMaximized = $state(false);

  async function syncMaximized() {
    try {
      isMaximized = await appWindow.isMaximized();
    } catch (error) {
      logError("Failed to check maximized state", error);
    }
  }

  onMount(() => {
    void syncMaximized();
    const unlisten = appWindow
      .onResized(syncMaximized)
      .catch((error) => logError("Failed to watch window size", error));
    return () => void unlisten.then((stop) => stop?.());
  });

  async function windowCall(action: "minimize" | "toggleMaximize" | "close") {
    try {
      // The capability grants maximize and unmaximize, not toggle-maximize.
      if (action === "toggleMaximize") await (isMaximized ? appWindow.unmaximize() : appWindow.maximize());
      else await appWindow[action]();
    } catch (error) {
      logError(`Window ${action} failed`, error);
    }
  }
</script>

<div class="ml-2 flex items-center gap-0.5 pr-1.5">
  <WindowControlButton label="Minimize" glyph="minimize" onclick={() => windowCall("minimize")} />
  <WindowControlButton
    label={isMaximized ? "Restore" : "Maximize"}
    glyph={isMaximized ? "restore" : "maximize"}
    onclick={() => windowCall("toggleMaximize")}
  />
  <WindowControlButton label="Close" glyph="close" onclick={() => windowCall("close")} />
</div>
