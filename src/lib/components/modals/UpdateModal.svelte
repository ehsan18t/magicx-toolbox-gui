<script lang="ts" module>
  // Lets the installer start before the app exits.
  const EXIT_AFTER_INSTALL_MS = 1000;
</script>

<script lang="ts">
  import { ExternalLink, Icon, MarkdownText } from "$lib/components/shared";
  import { Button, Callout, IconButton, Modal, SettingRow, Switch, textLink } from "$lib/components/ui";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { formatBytes, formatDate } from "$lib/utils/format";
  import { exit } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";

  const titleId = $props.id();

  const isOpen = $derived(modalStore.current === "update");
  const appVersion = $derived(appInfoStore.version);
  const isChecking = $derived(updateStore.isChecking);
  const isInstalling = $derived(updateStore.isInstalling);
  const updateInfo = $derived(updateStore.updateInfo);
  const error = $derived(updateStore.error);

  onMount(() => void appInfoStore.load());

  function checkForUpdate() {
    void updateStore.checkForUpdate(false);
  }

  function setIncludePrereleases(include: boolean) {
    settingsStore.setIncludePrereleases(include);
    // A result read under the old setting would offer the wrong release.
    if (updateInfo || isChecking) checkForUpdate();
  }

  async function installUpdate() {
    if (isInstalling || !updateInfo?.available) return;
    if (!(await updateStore.installUpdate())) return;
    setTimeout(async () => {
      try {
        await exit(0);
      } catch {
        // The installer is already running, and the backend keeps refusing applies until exit.
        modalStore.close();
        toastStore.warning(
          "The installer is running, but the app could not close itself. Close the app to finish the update.",
        );
      }
    }, EXIT_AFTER_INSTALL_MS);
  }

  const lastChecked = $derived.by(() => {
    const at = formatDate(settingsStore.lastUpdateCheck, { month: "long", time: "minutes" });
    return at ? `Last checked ${at}` : "Not checked yet";
  });
</script>

<Modal open={isOpen} onclose={modalStore.close} size="md" labelledBy={titleId}>
  <div class="relative overflow-y-auto px-7 pt-6 pb-5">
    <IconButton icon="mdi:close" label="Close" class="absolute top-3 right-3" onclick={modalStore.close} />

    <h2 id={titleId} class="m-0 font-display text-xl font-semibold">Updates</h2>

    <section class="mt-5" aria-live="polite">
      {#if updateInfo?.available}
        {@const released = formatDate(updateInfo.publishedAt, { month: "long" })}
        {@const size = updateInfo.assetSize ? formatBytes(updateInfo.assetSize) : ""}
        <div class="flex animate-fade-in flex-wrap items-baseline gap-x-2 gap-y-1">
          <p class="m-0 text-lg font-semibold">Version {updateInfo.latestVersion} is available</p>
          {#if updateInfo.prerelease}
            <span class="rounded border border-warning/40 bg-warning/10 px-1.5 py-px text-xs font-medium text-warning">
              Pre-release
            </span>
          {/if}
        </div>
        <p class="m-0 mt-1 text-ui text-foreground-muted">
          You have {appVersion || updateInfo.currentVersion}.
          {#if released}Released {released}.{/if}
          {#if size}{size} download.{/if}
        </p>

        {#if updateInfo.releaseNotes}
          <div class="mt-4 max-h-52 overflow-y-auto rounded-lg border border-border bg-card px-4 py-3 text-ui">
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
            <span class="text-ui text-foreground-muted">No installer for this PC in the release.</span>
          {/if}
          {#if updateInfo.downloadUrl}
            <ExternalLink href={updateInfo.downloadUrl} class="text-ui {textLink}">Download manually</ExternalLink>
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
            <p class="m-0 mt-1 text-ui text-foreground-muted">
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
        <Callout tone="error" icon="mdi:alert-circle" class="mt-4 animate-fade-in text-ui">
          <span class="min-w-0 flex-1">{error}</span>
          <button
            type="button"
            class="shrink-0 cursor-pointer text-xs text-foreground-muted underline hover:text-foreground"
            onclick={() => updateStore.clearError()}
          >
            Dismiss
          </button>
        </Callout>
      {/if}
    </section>

    <section class="mt-6 divide-y divide-border border-t border-border" aria-label="Update settings">
      <SettingRow
        title="Check for updates at startup"
        description="At most once an hour, in the background."
        density="flush"
      >
        <Switch
          checked={settingsStore.autoCheckUpdates}
          label="Check for updates at startup"
          onchange={(on) => settingsStore.setAutoCheckUpdates(on)}
        />
      </SettingRow>
      <SettingRow
        title="Include pre-releases"
        description="Also offer test builds marked pre-release on GitHub. They are newer but less tested."
        density="flush"
      >
        <Switch
          checked={settingsStore.includePrereleases}
          label="Include pre-releases"
          onchange={setIncludePrereleases}
        />
      </SettingRow>
      <SettingRow title="Install updates automatically" description="Not available yet." density="flush">
        <Switch checked={settingsStore.autoInstallUpdates} label="Install updates automatically" disabled />
      </SettingRow>
    </section>
  </div>
</Modal>
