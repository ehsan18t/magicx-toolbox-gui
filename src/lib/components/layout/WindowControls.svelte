<script lang="ts">
  import * as systemApi from "$lib/api/system";
  import { logError } from "$lib/utils/logger";
  import { onMount } from "svelte";
  import WindowControlButton from "./WindowControlButton.svelte";

  let isMaximized = $state(false);

  async function syncMaximized() {
    try {
      isMaximized = await systemApi.isWindowMaximized();
    } catch (error) {
      logError("Failed to check maximized state", error);
    }
  }

  onMount(() => {
    void syncMaximized();
    const unlisten = systemApi
      .onWindowResized(syncMaximized)
      .catch((error) => logError("Failed to watch window size", error));
    return () => void unlisten.then((stop) => stop?.());
  });

  async function windowCall(action: string, run: () => Promise<void>) {
    try {
      await run();
    } catch (error) {
      logError(`Window ${action} failed`, error);
    }
  }
</script>

<div class="ml-2 flex items-center gap-0.5 pr-1.5">
  <WindowControlButton
    label="Minimize"
    glyph="minimize"
    onclick={() => windowCall("minimize", systemApi.minimizeWindow)}
  />
  <WindowControlButton
    label={isMaximized ? "Restore" : "Maximize"}
    glyph={isMaximized ? "restore" : "maximize"}
    onclick={() => windowCall("maximize", () => systemApi.setWindowMaximized(!isMaximized))}
  />
  <WindowControlButton label="Close" glyph="close" onclick={() => windowCall("close", systemApi.closeWindow)} />
</div>
