<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { ColorSchemePicker } from "$lib/components/settings";
  import { Icon } from "$lib/components/shared";
  import { Button, SegmentedSwitch, SettingRow, Spinner, Switch, type SegmentOption } from "$lib/components/ui";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { logsStore } from "$lib/stores/logs.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { themeStore, type Theme } from "$lib/stores/theme.svelte";
  import { formatBytes, plural } from "$lib/utils/format";
  import { onMount, type Snippet } from "svelte";

  const THEMES: SegmentOption<Theme>[] = [
    { value: "light", label: "Light", icon: "tabler:sun" },
    { value: "dark", label: "Dark", icon: "tabler:moon" },
  ];

  const logs = $derived(logsStore.settings);

  onMount(() => void logsStore.loadSettings());

  async function deleteLogs() {
    const ok = await confirmStore.ask({
      title: "Delete saved logs?",
      message:
        "Deletes the log files saved on this PC, including this session's file. Files another running copy of MagicX Toolbox is still writing are kept.",
      confirmText: "Delete logs",
      variant: "danger",
    });
    if (ok) await logsStore.deleteLogs();
  }
</script>

{#snippet section(title: string, body: Snippet)}
  {@const id = `settings-${title.toLowerCase()}`}
  <section aria-labelledby={id}>
    <h2 {id} class="m-0 mb-2 text-sm font-semibold text-foreground">{title}</h2>
    <div class="divide-y divide-border rounded-lg border border-border bg-card">{@render body()}</div>
  </section>
{/snippet}

{#snippet appearance()}
  <SettingRow title="Theme">
    <SegmentedSwitch value={themeStore.current} options={THEMES} label="Theme" onchange={themeStore.set} />
  </SettingRow>
  <SettingRow title="Accent colour" description="Used for selections, applied states and progress.">
    <ColorSchemePicker size="md" />
  </SettingRow>
{/snippet}

{#snippet tweaks()}
  <SettingRow
    title="Show tweaks this PC cannot run"
    description="List tweaks and apps made for other Windows versions, greyed out as unavailable."
  >
    <Switch
      checked={settingsStore.showUnsupported}
      label="Show tweaks this PC cannot run"
      onchange={(show) => settingsStore.setShowUnsupported(show)}
    />
  </SettingRow>
{/snippet}

{#snippet diagnostics()}
  {#if logs}
    <SettingRow
      title="Save logs on this PC"
      description="Off keeps logs only until you close the app. Nothing is uploaded."
    >
      <Switch
        checked={logs.persist}
        loading={logsStore.isSettingsBusy}
        label="Save logs on this PC"
        onchange={(persist) => logsStore.setSettings(persist, logs.detailed)}
      />
    </SettingRow>
    <SettingRow
      title="Detailed logging"
      description="Records more steps, including script output. Turn on when asked; files grow faster."
    >
      <Switch
        checked={logs.detailed}
        loading={logsStore.isSettingsBusy}
        label="Detailed logging"
        onchange={(detailed) => logsStore.setSettings(logs.persist, detailed)}
      />
    </SettingRow>
    <div class="space-y-2.5 px-4 py-3.5">
      <div class="text-xs text-foreground-muted">
        <p class="m-0 font-mono break-all text-foreground select-text">{logs.folder || "Logs folder unavailable"}</p>
        <p class="m-0 mt-0.5">
          {plural(logs.files, "file")}, {formatBytes(logs.bytes)}{logs.persist ? "" : ". Existing log files are kept."}
        </p>
      </div>
      {#if logs.error}<p class="m-0 animate-fade-in text-xs text-error">Logging problem: {logs.error}</p>{/if}
      <div class="flex flex-wrap gap-2">
        <Button size="sm" onclick={() => logsStore.openFolder()}>
          <Icon icon="mdi:folder-open" size="md" />
          Open folder
        </Button>
        <Button size="sm" loading={logsStore.isExporting} onclick={() => logsStore.exportDiagnostics()}>
          {#if !logsStore.isExporting}<Icon icon="mdi:export" size="md" />{/if}
          Export diagnostics
        </Button>
        <Button size="sm" class="text-error" disabled={logsStore.isSettingsBusy} onclick={deleteLogs}>
          <Icon icon="mdi:delete-outline" size="md" />
          Delete logs
        </Button>
      </div>
    </div>
  {:else}
    <div class="flex justify-center py-3"><Spinner size="sm" label="Loading log settings" /></div>
  {/if}
{/snippet}

<PageLayout title="Settings" description="How MagicX Toolbox looks, which tweaks it lists, and what it logs.">
  <div class="mt-2 flex flex-col gap-6">
    {@render section("Appearance", appearance)}
    {@render section("Tweaks", tweaks)}
    {@render section("Diagnostics", diagnostics)}
  </div>
</PageLayout>
