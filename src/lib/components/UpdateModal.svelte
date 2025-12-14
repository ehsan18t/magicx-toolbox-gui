<script lang="ts">
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { exit } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";
  import { Button, IconButton, Modal, ModalBody, ModalHeader, Switch } from "./ui";

  let appVersion = $state("1.0.0");

  const isOpen = $derived(modalStore.current === "update");

  const isChecking = $derived(updateStore.isChecking);
  const isInstalling = $derived(updateStore.isInstalling);
  const updateInfo = $derived(updateStore.updateInfo);
  const error = $derived(updateStore.error);

  let autoCheckUpdates = $state(true);
  let autoInstallUpdates = $state(false);

  $effect(() => {
    const settings = settingsStore.settings;
    autoCheckUpdates = settings.autoCheckUpdates;
    autoInstallUpdates = settings.autoInstallUpdates;
  });

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (err) {
      console.error("Failed to get app version:", err);
    }
  });

  function handleAutoCheckToggle() {
    autoCheckUpdates = !autoCheckUpdates;
    settingsStore.setAutoCheckUpdates(autoCheckUpdates);
  }

  function handleAutoInstallToggle() {
    autoInstallUpdates = !autoInstallUpdates;
    settingsStore.setAutoInstallUpdates(autoInstallUpdates);
  }

  async function checkForUpdate() {
    if (isChecking) return;
    updateStore.clearError();
    const result = await updateStore.checkForUpdate(false);
    if (result) {
      settingsStore.setLastUpdateCheck(new Date().toISOString());
    }
  }

  async function installUpdate() {
    if (isInstalling || !updateInfo?.available) return;
    const success = await updateStore.installUpdate();
    if (success) {
      setTimeout(async () => {
        try {
          await exit(0);
        } catch {
          // Exit failed - inform user to restart manually
          closeModal();
          toastStore.warning("Update downloaded. Please restart the app manually to apply it.");
        }
      }, 1000);
    }
  }

  function formatDate(dateString: string | undefined | null): string {
    if (!dateString) return "Unknown";
    try {
      return new Date(dateString).toLocaleDateString(undefined, {
        year: "numeric",
        month: "long",
        day: "numeric",
      });
    } catch {
      return dateString;
    }
  }

  function formatBytes(bytes: number | undefined): string {
    if (!bytes) return "";
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }
</script>

<Modal open={isOpen} onclose={closeModal} size="md">
  <ModalHeader>
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
        <Icon icon="mdi:update" width="24" class="text-accent" />
      </div>
      <div>
        <h2 class="m-0 text-lg font-bold text-foreground">Updates</h2>
        <span class="text-sm text-foreground-muted">Current: v{appVersion}</span>
      </div>
    </div>
    <IconButton icon="mdi:close" onclick={closeModal} aria-label="Close" />
  </ModalHeader>

  <ModalBody class="space-y-5">
    {#if error}
      <div class="flex items-start gap-2 rounded-lg bg-error/15 p-3 text-error">
        <Icon icon="mdi:alert-circle" width="18" class="mt-0.5 shrink-0" />
        <div class="flex-1">
          <span class="text-sm">{error}</span>
          <button class="ml-2 text-xs underline opacity-70 hover:opacity-100" onclick={() => updateStore.clearError()}>
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <div class="rounded-lg border border-border bg-surface p-4">
      {#if updateInfo?.available}
        <div class="mb-4 flex items-start gap-3">
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-success/15">
            <Icon icon="mdi:arrow-up-circle" width="24" class="text-success" />
          </div>
          <div class="flex-1">
            <h3 class="m-0 text-base font-semibold text-foreground">Update Available!</h3>
            <p class="m-0 mt-1 text-sm text-foreground-muted">
              Version {updateInfo.latestVersion} is available
              {#if updateInfo.publishedAt}
                (released {formatDate(updateInfo.publishedAt)})
              {/if}
            </p>
            {#if updateInfo.assetSize}
              <p class="m-0 mt-0.5 text-xs text-foreground-subtle">
                Download size: {formatBytes(updateInfo.assetSize)}
              </p>
            {/if}
          </div>
        </div>

        {#if updateInfo.releaseNotes}
          <div class="bg-muted/50 mb-4 max-h-32 overflow-y-auto rounded-lg p-3">
            <h4 class="m-0 mb-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase">Release Notes</h4>
            <p class="m-0 text-sm whitespace-pre-wrap text-foreground">
              {updateInfo.releaseNotes}
            </p>
          </div>
        {/if}

        <div class="flex gap-2">
          {#if updateInfo.downloadUrl && updateInfo.assetName}
            <Button class="flex-1" onclick={installUpdate} disabled={isInstalling}>
              {#if isInstalling}
                <Icon icon="mdi:loading" width="18" class="animate-spin" />
                Downloading...
              {:else}
                <Icon icon="mdi:download" width="18" />
                Install Update
              {/if}
            </Button>
          {:else}
            <span class="flex-1 text-center text-sm text-foreground-muted">No compatible installer found</span>
          {/if}
          {#if updateInfo.downloadUrl}
            <ExternalLink
              href={updateInfo.downloadUrl}
              class="bg-muted hover:bg-muted/80 flex cursor-pointer items-center justify-center gap-2 rounded-lg border border-border px-4 py-2.5 text-sm font-medium text-foreground transition-colors"
              title="Download manually"
            >
              <Icon icon="mdi:open-in-new" width="16" />
            </ExternalLink>
          {/if}
        </div>
      {:else if updateInfo}
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-full bg-success/15">
            <Icon icon="mdi:check-circle" width="24" class="text-success" />
          </div>
          <div>
            <h3 class="m-0 text-base font-semibold text-foreground">You're up to date!</h3>
            <p class="m-0 mt-1 text-sm text-foreground-muted">
              Version {appVersion} is the latest version.
            </p>
          </div>
        </div>
      {:else}
        <div class="flex items-center gap-3">
          <div class="bg-muted flex h-10 w-10 items-center justify-center rounded-full">
            <Icon icon="mdi:help-circle" width="24" class="text-foreground-muted" />
          </div>
          <div>
            <h3 class="m-0 text-base font-semibold text-foreground">Check for updates</h3>
            <p class="m-0 mt-1 text-sm text-foreground-muted">Click the button below to check for new versions.</p>
          </div>
        </div>
      {/if}
    </div>

    {#if !updateInfo?.available}
      <Button variant="secondary" class="w-full" onclick={checkForUpdate} disabled={isChecking}>
        {#if isChecking}
          <Icon icon="mdi:loading" width="18" class="animate-spin" />
          Checking for updates...
        {:else}
          <Icon icon="mdi:refresh" width="18" />
          Check for Updates
        {/if}
      </Button>
    {/if}

    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-4 flex items-center gap-2 text-sm font-semibold text-foreground">
        <Icon icon="mdi:cog" width="18" class="text-accent" />
        Update Settings
      </h3>

      <div class="space-y-4">
        <label class="flex cursor-pointer items-center justify-between">
          <div class="flex-1">
            <span class="block text-sm font-medium text-foreground">Automatically check for updates</span>
            <span class="block text-xs text-foreground-muted">Check for updates when the app starts</span>
          </div>
          <Switch checked={autoCheckUpdates} onchange={handleAutoCheckToggle} />
        </label>

        <label class="flex cursor-pointer items-center justify-between">
          <div class="flex-1">
            <span class="block text-sm font-medium text-foreground">Automatically install updates</span>
            <span class="block text-xs text-foreground-muted"
              >Download and install updates automatically (coming soon)</span
            >
          </div>
          <Switch checked={autoInstallUpdates} onchange={handleAutoInstallToggle} disabled />
        </label>
      </div>
    </div>

    {#if settingsStore.lastUpdateCheck}
      <p class="m-0 text-center text-xs text-foreground-subtle">
        Last checked: {formatDate(settingsStore.lastUpdateCheck)}
      </p>
    {/if}
  </ModalBody>
</Modal>
