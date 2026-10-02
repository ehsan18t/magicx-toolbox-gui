<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { ColorSchemePicker } from "$lib/components/settings";
  import { Icon } from "$lib/components/shared";
  import { Button, SegmentedSwitch, Spinner, Switch } from "$lib/components/ui";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { logsStore } from "$lib/stores/logs.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";

  const logs = $derived(logsStore.settings);
  const themes = [
    { value: 0, label: "Light", icon: "tabler:sun" },
    { value: 1, label: "Dark", icon: "tabler:moon" },
  ];

  onMount(() => void logsStore.loadSettings());

  function storedSize(files: number, bytes: number): string {
    const size = bytes >= 1024 * 1024 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB` : `${Math.ceil(bytes / 1024)} KB`;
    return `${files} ${files === 1 ? "file" : "files"}, ${size}`;
  }

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
  <section aria-label={title}>
    <h2 class="m-0 mb-2 text-sm font-semibold text-foreground">{title}</h2>
    <div class="divide-y divide-border rounded-lg border border-border bg-card">{@render body()}</div>
  </section>
{/snippet}

{#snippet row(title: string, description: string, control: Snippet)}
  <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-3.5">
    <div class="min-w-0 flex-1 basis-64">
      <p class="m-0 text-ui font-medium">{title}</p>
      {#if description}<p class="m-0 mt-0.5 text-xs text-foreground-muted">{description}</p>{/if}
    </div>
    <div class="flex shrink-0 flex-wrap items-center gap-2">{@render control()}</div>
  </div>
{/snippet}

{#snippet appearance()}
  {#snippet themeControl()}
    <SegmentedSwitch
      value={themeStore.current === "dark" ? 1 : 0}
      options={themes}
      label="Theme"
      onchange={(v) => themeStore.set(v === 1 ? "dark" : "light")}
    />
  {/snippet}
  {#snippet accentControl()}<ColorSchemePicker size="md" />{/snippet}
  {@render row("Theme", "", themeControl)}
  {@render row("Accent colour", "Used for selections, applied states and progress.", accentControl)}
{/snippet}

{#snippet tweaks()}
  {#snippet unsupportedControl()}
    <Switch
      checked={settingsStore.showUnsupported}
      label="Show tweaks this PC cannot run"
      onchange={(show) => settingsStore.setShowUnsupported(show)}
    />
  {/snippet}
  {@render row(
    "Show tweaks this PC cannot run",
    "List tweaks and apps made for other Windows versions, greyed out as unavailable.",
    unsupportedControl,
  )}
{/snippet}

{#snippet diagnostics()}
  {#if logs}
    {#snippet persistControl()}
      <Switch
        checked={logs.persist}
        loading={logsStore.isSettingsBusy}
        label="Save logs on this PC"
        onchange={(persist) => logsStore.setSettings(persist, logs.detailed)}
      />
    {/snippet}
    {#snippet detailedControl()}
      <Switch
        checked={logs.detailed}
        loading={logsStore.isSettingsBusy}
        label="Detailed logging"
        onchange={(detailed) => logsStore.setSettings(logs.persist, detailed)}
      />
    {/snippet}
    {@render row(
      "Save logs on this PC",
      "Off keeps logs only until you close the app. Nothing is uploaded.",
      persistControl,
    )}
    {@render row(
      "Detailed logging",
      "Records more steps, including script output. Turn on when asked; files grow faster.",
      detailedControl,
    )}
    <div class="space-y-2.5 px-4 py-3.5">
      <div class="text-xs text-foreground-muted">
        <p class="m-0 font-mono break-all text-foreground select-text">{logs.folder || "Logs folder unavailable"}</p>
        <p class="m-0 mt-0.5">
          {storedSize(logs.files, logs.bytes)}{logs.persist ? "" : ". Existing log files are kept."}
        </p>
      </div>
      {#if logs.error}<p class="m-0 animate-fade-in text-xs text-error">Logging problem: {logs.error}</p>{/if}
      <div class="flex flex-wrap gap-2">
        <Button variant="secondary" size="sm" onclick={() => logsStore.openFolder()}>
          <Icon icon="mdi:folder-open" width="16" />
          Open folder
        </Button>
        <Button
          variant="secondary"
          size="sm"
          loading={logsStore.isExporting}
          onclick={() => logsStore.exportDiagnostics()}
        >
          {#if !logsStore.isExporting}<Icon icon="mdi:export" width="16" />{/if}
          Export diagnostics
        </Button>
        <Button
          variant="secondary"
          size="sm"
          class="text-error"
          disabled={logsStore.isSettingsBusy}
          onclick={deleteLogs}
        >
          <Icon icon="mdi:delete-outline" width="16" />
          Delete logs
        </Button>
      </div>
    </div>
  {:else}
    <div class="flex justify-center py-3"><Spinner size="sm" /></div>
  {/if}
{/snippet}

<PageLayout title="Settings" description="How MagicX Toolbox looks, which tweaks it lists, and what it logs.">
  <div class="mt-2 flex flex-col gap-6">
    {@render section("Appearance", appearance)}
    {@render section("Tweaks", tweaks)}
    {@render section("Diagnostics", diagnostics)}
  </div>
</PageLayout>
