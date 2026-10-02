<script lang="ts">
  import { ExternalLink, Icon, MarkdownText } from "$lib/components/shared";
  import { Button, Modal, Switch } from "$lib/components/ui";
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { exit } from "@tauri-apps/plugin-process";
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";

  let appVersion = $state("");

  const isOpen = $derived(modalStore.current === "update");
  const isChecking = $derived(updateStore.isChecking);
  const isInstalling = $derived(updateStore.isInstalling);
  const updateInfo = $derived(updateStore.updateInfo);
  const error = $derived(updateStore.error);

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (err) {
      console.error("Failed to get app version:", err);
    }
  });

  async function checkForUpdate() {
    if (isChecking) return;
    updateStore.clearError();
    const result = await updateStore.checkForUpdate(false);
    if (result) settingsStore.setLastUpdateCheck(new Date().toISOString());
  }

  function setIncludePrereleases(include: boolean) {
    settingsStore.setIncludePrereleases(include);
    // A result read under the old setting would offer the wrong release.
    if (updateInfo) void checkForUpdate();
  }

  async function installUpdate() {
    if (isInstalling || !updateInfo?.available) return;
    const success = await updateStore.installUpdate();
    if (success) {
      setTimeout(async () => {
        try {
          await exit(0);
        } catch {
          // The installer is already running, and the backend keeps refusing applies until exit.
          closeModal();
          toastStore.warning(
            "The installer is running, but the app could not close itself. Close the app to finish the update.",
          );
        }
      }, 1000);
    }
  }

  function formatDate(dateString: string | undefined | null): string {
    if (!dateString) return "";
    const date = new Date(dateString);
    return Number.isNaN(date.getTime())
      ? dateString
      : date.toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" });
  }

  const formatBytes = (bytes: number) => `${(bytes / (1024 * 1024)).toFixed(1)} MB`;

  function formatDateTime(dateString: string): string {
    const date = new Date(dateString);
    return Number.isNaN(date.getTime())
      ? dateString
      : date.toLocaleString(undefined, { dateStyle: "long", timeStyle: "short" });
  }

  const lastChecked = $derived(
    settingsStore.lastUpdateCheck ? `Last checked ${formatDateTime(settingsStore.lastUpdateCheck)}` : "Not checked yet",
  );
</script>

{#snippet row(title: string, description: string, control: Snippet)}
  <div class="flex items-center justify-between gap-6 py-3">
    <div class="min-w-0">
      <p class="m-0 text-[13px] font-medium">{title}</p>
      <p class="m-0 mt-0.5 text-xs text-foreground-muted">{description}</p>
    </div>
    {@render control()}
  </div>
{/snippet}

<Modal open={isOpen} onclose={closeModal} size="md" labelledBy="update-title">
  <div class="relative overflow-y-auto px-7 pt-6 pb-5">
    <button
      type="button"
      class="absolute top-3 right-3 flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
      aria-label="Close"
      onclick={closeModal}
    >
      <Icon icon="mdi:close" width="18" />
    </button>

    <h2 id="update-title" class="m-0 font-display text-xl font-semibold">Updates</h2>

    <section class="mt-5" aria-live="polite">
      {#if updateInfo?.available}
        <div class="flex animate-fade-in flex-wrap items-baseline gap-x-2 gap-y-1">
          <p class="m-0 text-lg font-semibold">Version {updateInfo.latestVersion} is available</p>
          {#if updateInfo.prerelease}
            <span class="rounded border border-warning/40 bg-warning/10 px-1.5 py-px text-xs font-medium text-warning">
              Pre-release
            </span>
          {/if}
        </div>
        <p class="m-0 mt-1 text-[13px] text-foreground-muted">
          You have {appVersion || updateInfo.currentVersion}.
          {#if updateInfo.publishedAt}Released {formatDate(updateInfo.publishedAt)}.{/if}
          {#if updateInfo.assetSize}{formatBytes(updateInfo.assetSize)} download.{/if}
        </p>

        {#if updateInfo.releaseNotes}
          <div class="mt-4 max-h-52 overflow-y-auto rounded-lg border border-border bg-card px-4 py-3 text-[13px]">
            <MarkdownText content={updateInfo.releaseNotes} />
          </div>
        {/if}

        <div class="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2">
          {#if updateInfo.downloadUrl && updateInfo.assetName}
            <Button variant="primary" onclick={installUpdate} loading={isInstalling}>
              {#if !isInstalling}<Icon icon="mdi:download" width="18" />{/if}
              {isInstalling ? "Downloading…" : "Install update"}
            </Button>
          {:else}
            <span class="text-[13px] text-foreground-muted">No installer for this PC in the release.</span>
          {/if}
          {#if updateInfo.downloadUrl}
            <ExternalLink
              href={updateInfo.downloadUrl}
              class="text-[13px] font-medium underline decoration-foreground-subtle underline-offset-4 hover:text-accent hover:decoration-accent"
            >
              Download manually
            </ExternalLink>
          {/if}
        </div>
      {:else}
        <div class="flex animate-fade-in flex-wrap items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="m-0 flex items-center gap-2 text-lg font-semibold">
              {#if updateInfo}
                <Icon icon="mdi:check-circle" width="20" class="text-success" />
                You're up to date
              {:else}
                Version {appVersion}
              {/if}
            </p>
            <p class="m-0 mt-1 text-[13px] text-foreground-muted">
              {#if updateInfo}Version {appVersion || updateInfo.currentVersion}.{/if}
              {lastChecked}.
            </p>
          </div>
          <Button variant="secondary" onclick={checkForUpdate} loading={isChecking}>
            {#if !isChecking}<Icon icon="mdi:refresh" width="16" />{/if}
            {isChecking ? "Checking…" : updateInfo ? "Check again" : "Check for updates"}
          </Button>
        </div>
      {/if}

      {#if error}
        <div
          class="mt-4 flex animate-fade-in items-start gap-2 rounded-lg border border-error/30 bg-error/8 px-3 py-2.5 text-[13px]"
        >
          <Icon icon="mdi:alert-circle" width="16" class="mt-0.5 shrink-0 text-error" />
          <span class="min-w-0 flex-1">{error}</span>
          <button
            type="button"
            class="shrink-0 cursor-pointer text-xs text-foreground-muted underline hover:text-foreground"
            onclick={() => updateStore.clearError()}
          >
            Dismiss
          </button>
        </div>
      {/if}
    </section>

    <section class="mt-6 divide-y divide-border border-t border-border" aria-label="Update settings">
      {#snippet autoCheck()}
        <Switch
          checked={settingsStore.autoCheckUpdates}
          ariaLabel="Check for updates at startup"
          onchange={(on) => settingsStore.setAutoCheckUpdates(on)}
        />
      {/snippet}
      {#snippet prereleases()}
        <Switch
          checked={settingsStore.includePrereleases}
          ariaLabel="Include pre-releases"
          onchange={setIncludePrereleases}
        />
      {/snippet}
      {#snippet autoInstall()}
        <Switch
          checked={settingsStore.autoInstallUpdates}
          ariaLabel="Install updates automatically"
          disabled
          onchange={(on) => settingsStore.setAutoInstallUpdates(on)}
        />
      {/snippet}
      {@render row("Check for updates at startup", "At most once an hour, in the background.", autoCheck)}
      {@render row(
        "Include pre-releases",
        "Also offer test builds marked pre-release on GitHub. They are newer but less tested.",
        prereleases,
      )}
      {@render row("Install updates automatically", "Not available yet.", autoInstall)}
    </section>
  </div>
</Modal>
