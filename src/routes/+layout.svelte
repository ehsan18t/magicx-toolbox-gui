<script lang="ts">
  import "@/app.css";
  import AboutModal from "@/lib/components/AboutModal.svelte";
  import DebugPanel from "@/lib/components/DebugPanel.svelte";
  import SettingsModal from "@/lib/components/SettingsModal.svelte";
  import TitleBar from "@/lib/components/TitleBar.svelte";
  import UpdateModal from "@/lib/components/UpdateModal.svelte";
  import { colorSchemeStore } from "@/lib/stores/colorScheme";
  import { settingsStore } from "@/lib/stores/settings";
  import { themeStore } from "@/lib/stores/theme";
  import { updateStore } from "@/lib/stores/update";
  import { onMount } from "svelte";

  const { children } = $props();

  onMount(() => {
    themeStore.init();
    colorSchemeStore.init();

    // Perform silent background update check if enabled
    const settings = settingsStore.get();
    if (settings.autoCheckUpdates) {
      // Check if enough time has passed since last check (at least 1 hour)
      const lastCheck = settings.lastUpdateCheck;
      const now = Date.now();
      const oneHour = 60 * 60 * 1000;

      const shouldCheck = !lastCheck || now - new Date(lastCheck).getTime() > oneHour;

      if (shouldCheck) {
        // Silent check - don't show errors to user
        updateStore.checkForUpdate(true).then((result) => {
          if (result) {
            settingsStore.setLastUpdateCheck(new Date().toISOString());
          }
        });
      }
    }
  });
</script>

<TitleBar />
<!-- TitleBar height=h-10 == 2.5rem -->
<main class="h-[calc(100dvh-2.5rem)] w-full overflow-auto">
  {@render children()}
</main>
<DebugPanel />

<!-- Global Modals -->
<AboutModal />
<SettingsModal />
<UpdateModal />
