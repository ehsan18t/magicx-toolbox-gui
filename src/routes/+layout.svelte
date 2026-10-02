<script lang="ts">
  import { showMainWindow } from "$lib/api/system";
  import { ApplyingOverlay, ToastContainer } from "$lib/components/feedback";
  import { LogsPanel, TitleBar } from "$lib/components/layout";
  import { AboutModal, ConfirmHost, ProfileExportModal, ProfileImportModal, UpdateModal } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { RETIRED_STORAGE_KEYS } from "$lib/config/app";
  import { colorSchemeStore } from "$lib/stores/colorScheme.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { initializeQuick } from "$lib/stores/tweaksData.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { errorMessage } from "$lib/utils/error";
  import { installErrorForwarding } from "$lib/utils/logger";
  import "@/app.css";
  import { onMount } from "svelte";

  let { children } = $props();

  let initError = $state<string | null>(null);

  // Global keyboard shortcuts
  function handleGlobalKeydown(e: KeyboardEvent) {
    // Ctrl+K or Cmd+K to focus search
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      navigationStore.focusSearch();
    }
  }

  onMount(async () => {
    installErrorForwarding();
    try {
      for (const key of RETIRED_STORAGE_KEYS) localStorage.removeItem(key);
    } catch {
      // Storage can be unavailable; the window must still be shown.
    }

    // Show the window now that the UI is ready
    try {
      await showMainWindow();
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
      initError = errorMessage(e);
      console.error("Failed to initialize categories:", e);
    }

    const initialLoader = document.getElementById("initial-loader");
    if (initialLoader) {
      initialLoader.addEventListener("animationend", (e) => e.target === initialLoader && initialLoader.remove());
      initialLoader.classList.add("fade-out");
    }

    if (initError) return;

    updateStore.autoCheckIfDue();
  });
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="flex h-dvh flex-col overflow-hidden">
  <TitleBar />
  <div class="min-h-0 flex-1">
    {#if initError}
      <div class="flex min-h-full items-center justify-center p-6">
        <div class="w-[min(92vw,420px)] rounded-lg border border-border bg-card p-6 text-center">
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
  </div>
  <!-- Otherwise the page docks it inside its content column, clear of the sidebar. -->
  {#if initError}<LogsPanel />{/if}
</div>

<AboutModal />
<UpdateModal />
<ProfileExportModal />
<ProfileImportModal />
<ConfirmHost />

<ApplyingOverlay />
<ToastContainer />
