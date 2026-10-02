<script lang="ts">
  import { ColorSchemePicker } from "$lib/components/settings";
  import { Icon } from "$lib/components/shared";
  import {
    Button,
    IconButton,
    Modal,
    ModalBody,
    ModalHeader,
    SegmentedSwitch,
    Spinner,
    Switch,
  } from "$lib/components/ui";
  import { logsStore } from "$lib/stores/logs.svelte";
  import {
    closeModal,
    modalStore,
    openProfileExportModal,
    openProfileImportModal,
    openUpdateModal,
  } from "$lib/stores/modal.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  const PROFILES_UNAVAILABLE = "Profiles are being rebuilt and are temporarily unavailable";

  let appVersion = $state("");
  let confirmingDelete = $state(false);

  const isOpen = $derived(modalStore.current === "settings");
  const logs = $derived(logsStore.settings);
  const themes = [
    { value: 0, label: "Light", icon: "tabler:sun" },
    { value: 1, label: "Dark", icon: "tabler:moon" },
  ];

  $effect(() => {
    if (isOpen) void logsStore.loadSettings();
  });

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  });

  function storedSize(files: number, bytes: number): string {
    const size = bytes >= 1024 * 1024 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB` : `${Math.ceil(bytes / 1024)} KB`;
    return `${files} ${files === 1 ? "file" : "files"}, ${size}`;
  }

  // Opening a second modal while this one animates out would stack two focus traps.
  function switchTo(open: () => void) {
    closeModal();
    setTimeout(open, 100);
  }
</script>

{#snippet section(title: string, body: Snippet)}
  <section>
    <h3 class="m-0 mb-2 text-[13px] font-semibold text-foreground">{title}</h3>
    <div class="divide-y divide-border rounded-lg border border-border bg-card">{@render body()}</div>
  </section>
{/snippet}

{#snippet row(title: string, description: string, control: Snippet)}
  <div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 px-3.5 py-3">
    <div class="min-w-0 flex-1 basis-56">
      <p class="m-0 text-[13px] font-medium">{title}</p>
      {#if description}<p class="m-0 mt-0.5 text-xs text-foreground-muted">{description}</p>{/if}
    </div>
    <div class="flex shrink-0 flex-wrap items-center gap-2">{@render control()}</div>
  </div>
{/snippet}

{#snippet appearance()}
  {#snippet themeControl()}
    <SegmentedSwitch
      value={themeStore.isDark ? 1 : 0}
      options={themes}
      onchange={(v) => themeStore.set(v === 1 ? "dark" : "light")}
    />
  {/snippet}
  {#snippet accentControl()}<ColorSchemePicker size="md" />{/snippet}
  {@render row("Theme", "", themeControl)}
  {@render row("Accent colour", "Used for selections, applied states and progress.", accentControl)}
{/snippet}
{#snippet updates()}
  {#snippet updateControl()}
    <Button variant="secondary" onclick={() => switchTo(openUpdateModal)}>
      <Icon icon="mdi:update" width="16" />
      Check for updates
    </Button>
  {/snippet}
  {@render row("Updates", "Automatic checks and installs are set in the update window.", updateControl)}
{/snippet}
{#snippet profiles()}
  {#snippet profileControl()}
    <Button variant="secondary" disabled title={PROFILES_UNAVAILABLE} onclick={() => switchTo(openProfileExportModal)}>
      <Icon icon="mdi:export" width="16" />
      Export
    </Button>
    <Button variant="secondary" disabled title={PROFILES_UNAVAILABLE} onclick={() => switchTo(openProfileImportModal)}>
      <Icon icon="mdi:import" width="16" />
      Import
    </Button>
  {/snippet}
  {@render row(
    "Configuration profiles",
    "Save your applied tweaks as a portable .mgx file to reuse after a reinstall or on another PC. Temporarily unavailable while profiles are rebuilt.",
    profileControl,
  )}
{/snippet}
{#snippet diagnostics()}
  {#if logs}
    {#snippet persistControl()}
      <Switch
        checked={logs.persist}
        loading={logsStore.settingsBusy}
        ariaLabel="Save logs on this PC"
        onchange={(persist) => logsStore.setSettings(persist, logs.detailed)}
      />
    {/snippet}
    {#snippet detailedControl()}
      <Switch
        checked={logs.detailed}
        loading={logsStore.settingsBusy}
        ariaLabel="Detailed logging"
        onchange={(detailed) => logsStore.setSettings(logs.persist, detailed)}
      />
    {/snippet}
    {#snippet fileControls()}
      <Button variant="secondary" size="sm" onclick={() => logsStore.openFolder()}>
        <Icon icon="mdi:folder-open" width="16" />
        Open folder
      </Button>
      <Button variant="secondary" size="sm" loading={logsStore.exporting} onclick={() => logsStore.exportDiagnostics()}>
        {#if !logsStore.exporting}<Icon icon="mdi:export" width="16" />{/if}
        Export diagnostics
      </Button>
      <Button
        variant="secondary"
        size="sm"
        class="text-error"
        disabled={logsStore.settingsBusy}
        onclick={() => (confirmingDelete = true)}
      >
        <Icon icon="mdi:delete-outline" width="16" />
        Delete logs
      </Button>
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
    <div class="space-y-2.5 px-3.5 py-3">
      <div class="text-xs text-foreground-muted">
        <p class="m-0 font-mono break-all text-foreground select-text">
          {logs.folder || "Logs folder unavailable"}
        </p>
        <p class="m-0 mt-0.5">
          {storedSize(logs.files, logs.bytes)}{logs.persist ? "" : ". Existing log files are kept."}
        </p>
      </div>
      {#if logs.error}<p class="m-0 text-xs text-error">Logging problem: {logs.error}</p>{/if}
      <div class="flex flex-wrap gap-2">{@render fileControls()}</div>
    </div>
  {:else}
    <div class="flex justify-center py-3"><Spinner size="sm" /></div>
  {/if}
{/snippet}

<Modal open={isOpen} onclose={closeModal} size="md" labelledBy="settings-modal-title">
  <ModalHeader>
    <h2 id="settings-modal-title" class="m-0 font-display text-xl font-semibold">Settings</h2>
    <IconButton icon="mdi:close" onclick={closeModal} aria-label="Close" />
  </ModalHeader>

  <ModalBody class="space-y-5">
    {@render section("Appearance", appearance)}

    {@render section("Updates", updates)}

    {@render section("Profiles", profiles)}

    {@render section("Diagnostics", diagnostics)}

    <p class="m-0 text-center text-xs text-foreground-subtle">MagicX Toolbox {appVersion}</p>
  </ModalBody>
</Modal>

<ConfirmDialog
  open={confirmingDelete}
  title="Delete saved logs?"
  message="Deletes the log files saved on this PC, including this session's file. Files another running copy of MagicX Toolbox is still writing are kept."
  confirmText="Delete logs"
  variant="danger"
  onconfirm={() => {
    confirmingDelete = false;
    void logsStore.deleteLogs();
  }}
  oncancel={() => (confirmingDelete = false)}
/>
