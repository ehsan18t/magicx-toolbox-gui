<script lang="ts">
  import { DebugPanel } from "$lib/components/debug";
  import { ApplyingOverlay, ToastContainer } from "$lib/components/feedback";
  import { TitleBar } from "$lib/components/layout";
  import {
    AboutModal,
    ProfileExportModal,
    ProfileImportModal,
    SettingsModal,
    TweakDetailsModal,
    UpdateModal,
  } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { colorSchemeStore } from "$lib/stores/colorScheme.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { initializeQuick } from "$lib/stores/tweaksData.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import "@/app.css";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  const { children } = $props();

  let initError = $state<string | null>(null);

  onMount(async () => {
    // Show the window now that the UI is ready
    try {
      await invoke("show_main_window");
    } catch (e) {
      console.error("Failed to show window:", e);
    }

    // Init theme stores (synchronous, fast)
    themeStore.init();
    colorSchemeStore.init();

    // Start data loading IMMEDIATELY (categories first - enables UI quickly)
    // CRITICAL: We await here to ensure +page.svelte has categories loaded
    // before its onMount runs. This prevents race conditions and simplifies page logic.
    try {
      await initializeQuick();
    } catch (e) {
      initError = e instanceof Error ? e.message : "Failed to initialize";
      console.error("Failed to initialize categories:", e);
    }

    // Hide the initial HTML loader now that Svelte is ready
    const initialLoader = document.getElementById("initial-loader");
    if (initialLoader) {
      initialLoader.classList.add("fade-out");
      setTimeout(() => initialLoader.remove(), 200);
    }

    if (initError) return;

    // Validate and clean up stale backup snapshots in background
    invoke("validate_snapshots")
      .then((removed) => {
        if (import.meta.env.DEV && removed && typeof removed === "number" && removed > 0) {
          console.log(`Cleaned up ${removed} stale backup snapshot(s)`);
        }
      })
      .catch((e) => {
        // Non-critical error, just log it in dev mode
        if (import.meta.env.DEV) {
          console.warn("Failed to validate snapshots:", e);
        }
      });

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
  {#if initError}
    <div class="flex min-h-full items-center justify-center p-6">
      <div class="w-[min(92vw,420px)] rounded-xl border border-border bg-card p-6 text-center">
        <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-error/15 text-error">
          <Icon icon="mdi:alert-circle" width="28" />
        </div>
        <h2 class="mt-4 mb-1 text-base font-semibold text-foreground">Failed to Load</h2>
        <p class="m-0 text-sm text-foreground-muted">{initError}</p>
        <button
          type="button"
          class="mt-5 inline-flex w-full items-center justify-center gap-2 rounded-lg bg-accent px-4 py-2.5 text-sm font-semibold text-accent-foreground transition-colors hover:bg-accent/90"
          onclick={() => window.location.reload()}
        >
          <Icon icon="mdi:refresh" width="18" />
          Retry
        </button>
      </div>
    </div>
  {:else}
    {@render children()}
  {/if}
</main>
<DebugPanel />

<!-- Global Modals -->
<AboutModal />
<SettingsModal />
<UpdateModal />
<TweakDetailsModal />
<ProfileExportModal />
<ProfileImportModal />

<ApplyingOverlay />
<ToastContainer />
