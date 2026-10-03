<script lang="ts">
  import * as systemApi from "$lib/api/system";
  import { ApplyingOverlay, LoadError, ToastContainer } from "$lib/components/feedback";
  import { LogsPanel, TitleBar } from "$lib/components/layout";
  import { AboutModal, ConfirmHost, ProfileExportModal, ProfileImportModal, UpdateModal } from "$lib/components/modals";
  import { RETIRED_STORAGE_KEYS } from "$lib/config/app";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { colorSchemeStore } from "$lib/stores/colorScheme.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { errorMessage } from "$lib/utils/error";
  import { installErrorForwarding, logError } from "$lib/utils/logger";
  import "@/app.css";
  import { onMount, type Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  let initError = $state<string | null>(null);

  function handleGlobalKeydown(e: KeyboardEvent) {
    // `code`, not `key`: a non-Latin layout reports another letter for the K key.
    if ((e.ctrlKey || e.metaKey) && e.code === "KeyK") {
      e.preventDefault();
      navigationStore.focusSearch();
    }
  }

  async function init() {
    installErrorForwarding();
    try {
      for (const key of RETIRED_STORAGE_KEYS) localStorage.removeItem(key);
    } catch {
      // Storage can be unavailable; the window must still be shown.
    }

    try {
      await systemApi.showMainWindow();
    } catch (error) {
      logError("Failed to show window", error);
    }

    themeStore.init();
    colorSchemeStore.init();
    void appInfoStore.load();

    try {
      await tweaksStore.load();
    } catch (error) {
      initError = errorMessage(error);
      logError("Failed to load the tweak model", error);
    }

    const initialLoader = document.getElementById("initial-loader");
    if (initialLoader) {
      initialLoader.addEventListener("animationend", (e) => e.target === initialLoader && initialLoader.remove());
      initialLoader.classList.add("fade-out");
    }

    if (!initError) updateStore.autoCheckIfDue();
  }

  onMount(() => {
    const closeBlocked = systemApi
      .onCloseBlocked((message) => toastStore.warning(message))
      .catch((error) => logError("Failed to listen for blocked closes", error));
    void init();
    return () => void closeBlocked.then((unlisten) => unlisten?.());
  });
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="flex h-dvh flex-col overflow-hidden">
  <TitleBar />
  <div class="min-h-0 flex-1">
    {#if initError}
      <div class="flex min-h-full items-center justify-center p-6">
        <LoadError message={initError} />
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
