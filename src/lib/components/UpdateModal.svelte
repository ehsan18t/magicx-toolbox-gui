<script lang="ts">
  import { closeModal, modalStore } from "$lib/stores/modal";
  import { settingsStore } from "$lib/stores/settings";
  import type { UpdateInfo } from "$lib/types";
  import { getVersion } from "@tauri-apps/api/app";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";

  let appVersion = $state("1.0.0");
  let isChecking = $state(false);
  let isInstalling = $state(false);
  let updateInfo = $state<UpdateInfo | null>(null);
  let error = $state<string | null>(null);

  const isOpen = $derived($modalStore === "update");

  // Get settings from store
  let autoCheckUpdates = $state(true);
  let autoInstallUpdates = $state(false);

  // Sync with settings store
  $effect(() => {
    const settings = $settingsStore;
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

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpen) {
      closeModal();
    }
  }

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
    isChecking = true;
    error = null;

    try {
      const result = await invoke<UpdateInfo>("check_for_update");
      updateInfo = result;
      settingsStore.setLastUpdateCheck(new Date().toISOString());
    } catch (err) {
      console.error("Update check failed:", err);
      error = err instanceof Error ? err.message : String(err);
      // Fallback: show current version as up to date
      updateInfo = {
        available: false,
        currentVersion: appVersion,
      };
    } finally {
      isChecking = false;
    }
  }

  async function installUpdate() {
    if (isInstalling || !updateInfo?.available) return;
    isInstalling = true;
    error = null;

    try {
      await invoke("install_update");
    } catch (err) {
      console.error("Update installation failed:", err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isInstalling = false;
    }
  }

  function formatDate(dateString: string | undefined): string {
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
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-1000 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="animate-in zoom-in-95 w-[min(90vw,500px)] rounded-xl border border-border bg-card shadow-xl duration-200"
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-title"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-border px-5 py-4">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
            <Icon icon="mdi:update" width="24" class="text-accent" />
          </div>
          <div>
            <h2 id="update-title" class="m-0 text-lg font-bold text-foreground">Updates</h2>
            <span class="text-sm text-foreground-muted">Current: v{appVersion}</span>
          </div>
        </div>
        <button
          class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-foreground-muted transition-colors hover:bg-[hsl(var(--muted))] hover:text-foreground"
          onclick={closeModal}
          aria-label="Close"
        >
          <Icon icon="mdi:close" width="20" />
        </button>
      </div>

      <!-- Body -->
      <div class="space-y-5 px-5 py-5">
        <!-- Error Message -->
        {#if error}
          <div class="flex items-center gap-2 rounded-lg bg-error/15 p-3 text-error">
            <Icon icon="mdi:alert-circle" width="18" />
            <span class="text-sm">{error}</span>
          </div>
        {/if}

        <!-- Update Status Section -->
        <div class="rounded-lg border border-border bg-surface p-4">
          {#if updateInfo?.available}
            <!-- Update Available -->
            <div class="mb-4 flex items-start gap-3">
              <div
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-success/15"
              >
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
              </div>
            </div>

            {#if updateInfo.releaseNotes}
              <div class="mb-4 max-h-32 overflow-y-auto rounded-lg bg-[hsl(var(--muted)/0.5)] p-3">
                <h4 class="m-0 mb-2 text-xs font-semibold text-foreground-muted uppercase">
                  Release Notes
                </h4>
                <p class="m-0 text-sm whitespace-pre-wrap text-foreground">
                  {updateInfo.releaseNotes}
                </p>
              </div>
            {/if}

            <div class="flex gap-2">
              <button
                class="flex flex-1 cursor-pointer items-center justify-center gap-2 rounded-lg border-0 bg-accent px-4 py-2.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90 disabled:cursor-not-allowed disabled:opacity-50"
                onclick={installUpdate}
                disabled={isInstalling}
              >
                {#if isInstalling}
                  <Icon icon="mdi:loading" width="18" class="animate-spin" />
                  Installing...
                {:else}
                  <Icon icon="mdi:download" width="18" />
                  Install Update
                {/if}
              </button>
              {#if updateInfo.downloadUrl}
                <ExternalLink
                  href={updateInfo.downloadUrl}
                  class="flex cursor-pointer items-center justify-center gap-2 rounded-lg border border-border bg-[hsl(var(--muted))] px-4 py-2.5 text-sm font-medium text-foreground transition-colors hover:bg-[hsl(var(--muted)/0.8)]"
                >
                  <Icon icon="mdi:open-in-new" width="16" />
                </ExternalLink>
              {/if}
            </div>
          {:else if updateInfo}
            <!-- Up to Date -->
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
            <!-- Not Checked -->
            <div class="flex items-center gap-3">
              <div
                class="flex h-10 w-10 items-center justify-center rounded-full bg-[hsl(var(--muted))]"
              >
                <Icon icon="mdi:help-circle" width="24" class="text-foreground-muted" />
              </div>
              <div>
                <h3 class="m-0 text-base font-semibold text-foreground">Check for updates</h3>
                <p class="m-0 mt-1 text-sm text-foreground-muted">
                  Click the button below to check for new versions.
                </p>
              </div>
            </div>
          {/if}
        </div>

        <!-- Check for Updates Button -->
        <button
          class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-lg border border-border bg-[hsl(var(--muted))] px-4 py-3 text-sm font-medium text-foreground transition-colors hover:bg-[hsl(var(--muted)/0.8)] disabled:cursor-not-allowed disabled:opacity-50"
          onclick={checkForUpdate}
          disabled={isChecking}
        >
          {#if isChecking}
            <Icon icon="mdi:loading" width="18" class="animate-spin" />
            Checking for updates...
          {:else}
            <Icon icon="mdi:refresh" width="18" />
            Check for Updates
          {/if}
        </button>

        <!-- Settings Section -->
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-4 flex items-center gap-2 text-sm font-semibold text-foreground">
            <Icon icon="mdi:cog" width="18" class="text-accent" />
            Update Settings
          </h3>

          <div class="space-y-4">
            <!-- Auto Check Updates -->
            <label class="flex cursor-pointer items-center justify-between">
              <div class="flex-1">
                <span class="block text-sm font-medium text-foreground">
                  Automatically check for updates
                </span>
                <span class="block text-xs text-foreground-muted">
                  Check for updates when the app starts
                </span>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={autoCheckUpdates}
                aria-label="Toggle automatic update checks"
                class="relative h-6 w-11 shrink-0 cursor-pointer rounded-full border-0 transition-colors {autoCheckUpdates
                  ? 'bg-accent'
                  : 'bg-[hsl(var(--muted))]'}"
                onclick={handleAutoCheckToggle}
              >
                <span
                  class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {autoCheckUpdates
                    ? 'translate-x-5'
                    : 'translate-x-0'}"
                ></span>
              </button>
            </label>

            <!-- Auto Install Updates -->
            <label class="flex cursor-pointer items-center justify-between">
              <div class="flex-1">
                <span class="block text-sm font-medium text-foreground">
                  Automatically install updates
                </span>
                <span class="block text-xs text-foreground-muted">
                  Download and install updates automatically
                </span>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={autoInstallUpdates}
                aria-label="Toggle automatic update installation"
                class="relative h-6 w-11 shrink-0 cursor-pointer rounded-full border-0 transition-colors {autoInstallUpdates
                  ? 'bg-accent'
                  : 'bg-[hsl(var(--muted))]'}"
                onclick={handleAutoInstallToggle}
              >
                <span
                  class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {autoInstallUpdates
                    ? 'translate-x-5'
                    : 'translate-x-0'}"
                ></span>
              </button>
            </label>
          </div>
        </div>

        <!-- Last Check Info -->
        {#if $settingsStore.lastUpdateCheck}
          <p class="m-0 text-center text-xs text-foreground-subtle">
            Last checked: {formatDate($settingsStore.lastUpdateCheck)}
          </p>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes zoom-in-95 {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .animate-in {
    animation: zoom-in-95 0.2s ease-out;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
