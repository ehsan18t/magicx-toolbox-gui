<script lang="ts">
  import "@/app.css";
  import AboutModal from "@/lib/components/AboutModal.svelte";
  import ApplyingOverlay from "@/lib/components/ApplyingOverlay.svelte";
  import DebugPanel from "@/lib/components/DebugPanel.svelte";
  import PendingRebootBanner from "@/lib/components/PendingRebootBanner.svelte";
  import SettingsModal from "@/lib/components/SettingsModal.svelte";
  import TitleBar from "@/lib/components/TitleBar.svelte";
  import ToastContainer from "@/lib/components/ToastContainer.svelte";
  import TweakDetailsModal from "@/lib/components/TweakDetailsModal.svelte";
  import UpdateModal from "@/lib/components/UpdateModal.svelte";
  import { colorSchemeStore } from "@/lib/stores/colorScheme.svelte";
  import { settingsStore } from "@/lib/stores/settings.svelte";
  import { themeStore } from "@/lib/stores/theme.svelte";
  import { updateStore } from "@/lib/stores/update.svelte";
  import { onMount } from "svelte";

  const { children } = $props();

  onMount(() => {
    themeStore.init();
    colorSchemeStore.init();

    // Perform silent background update check if enabled
    const settings = settingsStore.settings;
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
  <PendingRebootBanner />
  {@render children()}
</main>
<DebugPanel />

<!-- Global Modals -->
<AboutModal />
<SettingsModal />
<UpdateModal />
<TweakDetailsModal />

<ApplyingOverlay />
<ToastContainer />
