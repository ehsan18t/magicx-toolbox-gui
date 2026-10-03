<script lang="ts">
  import * as systemApi from "$lib/api/system";
  import { ApplyingOverlay, LoadError, ToastContainer } from "$lib/components/feedback";
  import { LogsPanel, TitleBar } from "$lib/components/layout";
  import { AboutModal, ConfirmHost, ProfileImportModal, UpdateModal } from "$lib/components/modals";
  import { RETIRED_STORAGE_KEYS } from "$lib/config/app";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { bootStore } from "$lib/stores/boot.svelte";
  import { colorSchemeStore } from "$lib/stores/colorScheme.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { installErrorForwarding, logError } from "$lib/utils/logger";
  import "@/app.css";
  import { onMount } from "svelte";
  import Workspace from "./Workspace.svelte";

  let shell = $state<HTMLElement | null>(null);

  function handleGlobalKeydown(e: KeyboardEvent) {
    // AltGr reports Ctrl+Alt, so its characters are not Ctrl+K.
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
    // A synthetic keydown can lack `key`.
    const key = e.key ?? "";
    // `key` follows the layout (Dvorak's K is not on KeyK); `code` only when the key is no Latin letter.
    const isK = key.toLowerCase() === "k" || (!/^[a-z]$/i.test(key) && e.code === "KeyK");
    if (!isK) return;
    e.preventDefault();
    // Not behind a dialog or the applying overlay: switching the page there would close or strand them.
    if (tweakActionsStore.isBusy || document.querySelector('[aria-modal="true"]')) return;
    navigationStore.focusSearch();
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

    await bootStore.load(() => {
      const initialLoader = document.getElementById("initial-loader");
      if (initialLoader) {
        initialLoader.addEventListener("animationend", (e) => e.target === initialLoader && initialLoader.remove());
        initialLoader.classList.add("fade-out");
      }
      if (!bootStore.error) updateStore.autoCheckIfDue();
    });
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

<div class="flex h-dvh flex-col overflow-hidden" bind:this={shell}>
  <TitleBar />
  <div class="min-h-0 flex-1">
    {#if bootStore.error}
      <LoadError message={bootStore.error} />
    {:else}
      <Workspace />
    {/if}
  </div>
  <!-- Otherwise the page docks it inside its content column, clear of the sidebar. -->
  {#if bootStore.error}<LogsPanel />{/if}
</div>

<AboutModal />
<UpdateModal />
<ProfileImportModal />
<ConfirmHost />

<ApplyingOverlay {shell} />
<ToastContainer />
