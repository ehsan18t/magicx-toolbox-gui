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
  import { TOAST_DURATION, toastStore } from "$lib/stores/toast.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { errorMessage } from "$lib/utils/error";
  import { logError } from "$lib/utils/logger";
  import "@/app.css";
  import { onMount } from "svelte";
  import Workspace from "./Workspace.svelte";

  const MODAL_HOSTS = [AboutModal, UpdateModal, ProfileImportModal, ConfirmHost];

  let shell = $state<HTMLElement | null>(null);
  let workspaceFailed = $state(false);

  function modalFailed(error: unknown, reset: () => void) {
    toastStore.failure("A dialog failed", error, {
      withContext: true,
      duration: TOAST_DURATION.long,
      action: { label: "Retry", run: reset },
    });
  }

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
    try {
      for (const key of RETIRED_STORAGE_KEYS) localStorage.removeItem(key);
    } catch {
      // Storage can be unavailable; the app must still start.
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

<!-- Catches the shell outside the inner boundaries: the title bar, the overlay and the toasts. -->
<svelte:boundary onerror={(error) => logError("The app failed to render", error)}>
  <div class="flex h-dvh flex-col overflow-hidden" bind:this={shell}>
    <TitleBar />
    <div class="min-h-0 flex-1">
      {#if bootStore.error}
        <LoadError message={bootStore.error} />
      {:else}
        <svelte:boundary
          onerror={(error) => {
            workspaceFailed = true;
            logError("Workspace failed to render", error);
          }}
        >
          <Workspace />
          {#snippet failed(error, reset)}
            <LoadError
              title="Something went wrong"
              message={errorMessage(error)}
              onretry={() => {
                workspaceFailed = false;
                reset();
              }}
            />
          {/snippet}
        </svelte:boundary>
      {/if}
    </div>
    <!-- Otherwise the page docks it inside its content column, clear of the sidebar. -->
    {#if bootStore.error || workspaceFailed}<LogsPanel />{/if}
  </div>

  <!-- One boundary per host: a broken dialog must not take down the shell or the other dialogs. -->
  {#each MODAL_HOSTS as Host (Host)}
    <svelte:boundary onerror={modalFailed}>
      <Host />
    </svelte:boundary>
  {/each}

  <ApplyingOverlay {shell} />
  <ToastContainer />

  {#snippet failed(error, reset)}
    <!-- The title bar may be what failed, so the window closes from here. -->
    <div class="flex h-dvh flex-col">
      <LoadError title="Something went wrong" message={errorMessage(error)} onretry={reset} closable />
    </div>
  {/snippet}
</svelte:boundary>
