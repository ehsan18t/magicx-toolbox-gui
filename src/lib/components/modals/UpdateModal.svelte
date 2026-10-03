<script lang="ts">
  import { Icon, MarkdownText } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Callout,
    Card,
    ExternalLink,
    LinkButton,
    Modal,
    ModalBody,
    ModalHeader,
    ProgressBar,
    SettingRow,
    Switch,
    WIDE_DIALOG_INSET,
  } from "$lib/components/ui";
  import { APP_CONFIG } from "$lib/config/app";
  import { HEADING } from "$lib/design";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { UPDATE_CHECK_CADENCE, updateStore } from "$lib/stores/update.svelte";
  import { versionLabel } from "$lib/stores/diagnostics";
  import { formatBytes } from "$lib/utils/format";
  import { formatDate } from "$lib/utils/time";
  import { onMount } from "svelte";

  const isOpen = $derived(modalStore.current === "update");
  const appVersion = $derived(appInfoStore.version);
  const isChecking = $derived(updateStore.isChecking);
  const isInstalling = $derived(updateStore.isInstalling);
  const updateInfo = $derived(updateStore.updateInfo);
  const error = $derived(updateStore.error);
  const download = $derived(updateStore.downloadProgress);
  const downloadTotal = $derived(download?.total ?? 0);

  onMount(() => void appInfoStore.load());

  function checkForUpdate() {
    void updateStore.checkForUpdate(false);
  }

  function setIncludePrereleases(include: boolean) {
    settingsStore.setIncludePrereleases(include);
    // A result read under the old setting would offer the wrong release.
    if (updateInfo || isChecking) checkForUpdate();
  }

  const lastChecked = $derived.by(() => {
    const at = formatDate(settingsStore.lastUpdateCheck, { month: "long", time: "minutes" });
    return at ? `Last checked ${at}` : "Not checked yet";
  });

  // Read out on its own: a live region around the result would read the release notes too.
  const status = $derived.by(() => {
    if (isChecking) return "Checking for updates";
    if (error) return error;
    if (updateInfo?.available) return `Version ${updateInfo.latestVersion} is available`;
    return updateInfo ? "You're up to date" : "";
  });
</script>

<Modal open={isOpen} onclose={modalStore.close} size="md">
  <ModalHeader title="Updates" size="xl" floating class="{WIDE_DIALOG_INSET} pt-6 pb-0" onclose={modalStore.close} />

  <ModalBody class={WIDE_DIALOG_INSET}>
    <p class="sr-only" aria-live="polite">{status}</p>
    <div>
      {#if updateInfo?.available}
        {@const released = formatDate(updateInfo.publishedAt, { month: "long" })}
        {@const size = updateInfo.assetSize ? formatBytes(updateInfo.assetSize) : ""}
        <div class="flex animate-fade-in flex-wrap items-baseline gap-x-2 gap-y-1">
          <p class={["m-0", HEADING.status]}>Version {updateInfo.latestVersion} is available</p>
          {#if updateInfo.prerelease}
            <Badge tone="warning">Pre-release</Badge>
          {/if}
        </div>
        <p class="m-0 mt-1 text-ui text-foreground-muted">
          You have {appVersion || updateInfo.currentVersion}.
          {#if released}Released {released}.{/if}
          {#if size}{size} download.{/if}
        </p>

        {#if updateInfo.releaseNotes}
          <Card class="mt-4 max-h-52 overflow-y-auto px-4 py-3 text-ui">
            <MarkdownText content={updateInfo.releaseNotes} />
          </Card>
        {/if}

        <div class="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2">
          {#if updateInfo.downloadUrl && updateInfo.assetName}
            <Button
              variant="primary"
              icon="mdi:download"
              loading={isInstalling}
              onclick={() => updateStore.installUpdate()}
            >
              {isInstalling ? "Downloading…" : "Install update"}
            </Button>
          {:else}
            <span class="text-ui text-foreground-muted">This release has no portable exe to update from.</span>
          {/if}
          {#if updateInfo.downloadUrl}
            <ExternalLink href={updateInfo.downloadUrl} variant="underline" class="text-ui"
              >Download manually</ExternalLink
            >
          {/if}
        </div>

        {#if isInstalling}
          <ProgressBar
            class="mt-4 animate-fade-in"
            value={download && downloadTotal > 0 ? download.downloaded : null}
            max={downloadTotal}
            label="Downloading the update"
            showValue
          />
        {/if}
      {:else}
        <div class="animate-fade-in">
          <div class="flex items-center justify-between gap-4">
            <p class={["m-0 flex min-w-0 items-center gap-2", HEADING.status]}>
              {#if updateInfo}
                <Icon icon="mdi:check-circle" size="xl" class="text-success" />
                You're up to date
              {:else}
                {versionLabel()}
              {/if}
            </p>
            <Button
              variant="secondary"
              icon="mdi:refresh"
              class="shrink-0"
              loading={isChecking}
              onclick={checkForUpdate}
            >
              {isChecking ? "Checking…" : updateInfo ? "Check again" : "Check for updates"}
            </Button>
          </div>
          <p class="m-0 mt-1 text-ui text-foreground-muted">
            {#if updateInfo}Version {appVersion || updateInfo.currentVersion}.{/if}
            {lastChecked}.
          </p>
        </div>
      {/if}

      {#if error}
        <Callout tone="error" icon="mdi:alert-circle" class="mt-4 animate-fade-in text-ui">
          <span class="min-w-0 flex-1">
            {error}
            {#if updateStore.manualDownloadOnly}
              <ExternalLink href="{APP_CONFIG.githubRepo}/releases" variant="underline"
                >Open the releases page</ExternalLink
              >
            {/if}
          </span>
          <LinkButton tone="muted" class="shrink-0 text-xs" onclick={() => updateStore.clearError()}>
            Dismiss
          </LinkButton>
        </Callout>
      {/if}
    </div>

    <section class="mt-6 divide-y divide-border border-t border-border" aria-label="Update settings">
      <SettingRow
        title="Check for updates at startup"
        description="At most {UPDATE_CHECK_CADENCE}, in the background."
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
        <Switch checked={false} label="Install updates automatically" disabled />
      </SettingRow>
    </section>
  </ModalBody>
</Modal>
