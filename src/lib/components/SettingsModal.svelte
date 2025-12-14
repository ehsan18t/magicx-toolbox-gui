<script lang="ts">
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { tweaksStore } from "$lib/stores/tweaks.svelte";
  import type { ExportData, TweakSnapshot } from "$lib/types";
  import { getVersion } from "@tauri-apps/api/app";
  import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import { Button, IconButton, Modal, ModalBody, ModalHeader } from "./ui";

  let appVersion = $state("1.0.0");
  let isExporting = $state(false);
  let isImporting = $state(false);
  let statusMessage = $state<{ type: "success" | "error"; text: string } | null>(null);

  const isOpen = $derived(modalStore.current === "settings");

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  });

  function showStatus(type: "success" | "error", text: string) {
    statusMessage = { type, text };
    setTimeout(() => {
      statusMessage = null;
    }, 3000);
  }

  async function createTweakSnapshots(): Promise<TweakSnapshot[]> {
    const tweaks = tweaksStore.list;
    const snapshots: TweakSnapshot[] = [];

    for (const tweak of tweaks) {
      snapshots.push({
        tweakId: tweak.definition.id,
        tweakName: tweak.definition.name,
        isApplied: tweak.status.is_applied,
        registryValues: {},
        snapshotTime: new Date().toISOString(),
      });
    }

    return snapshots;
  }

  async function handleExport() {
    if (isExporting) return;
    isExporting = true;

    try {
      const snapshots = await createTweakSnapshots();
      const settings = settingsStore.settings;

      const exportData: ExportData = {
        version: "1.0",
        exportTime: new Date().toISOString(),
        appVersion,
        settings,
        tweakSnapshots: snapshots,
      };

      const filePath = await save({
        defaultPath: `magicx-backup-${new Date().toISOString().split("T")[0]}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
        showStatus("success", "Settings exported successfully!");
      }
    } catch (error) {
      console.error("Export failed:", error);
      showStatus("error", `Export failed: ${error}`);
    } finally {
      isExporting = false;
    }
  }

  async function handleImport() {
    if (isImporting) return;
    isImporting = true;

    try {
      const filePath = await openDialog({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath && typeof filePath === "string") {
        const content = await readTextFile(filePath);
        const importData = JSON.parse(content) as ExportData;

        if (!importData.version || !importData.settings) {
          throw new Error("Invalid backup file format");
        }

        // Validate settings schema to prevent malformed data
        const settings = importData.settings;
        if (typeof settings.autoCheckUpdates !== "boolean" || typeof settings.autoInstallUpdates !== "boolean") {
          throw new Error("Invalid settings format in backup file");
        }

        settingsStore.update(settings);
        showStatus("success", `Imported ${importData.tweakSnapshots.length} tweak snapshots!`);
      }
    } catch (error) {
      console.error("Import failed:", error);
      showStatus("error", `Import failed: ${error}`);
    } finally {
      isImporting = false;
    }
  }
</script>

<Modal open={isOpen} onclose={closeModal} size="md">
  <ModalHeader>
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
        <Icon icon="mdi:cog" width="24" class="text-accent" />
      </div>
      <h2 class="m-0 text-lg font-bold text-foreground">Settings</h2>
    </div>
    <IconButton icon="mdi:close" onclick={closeModal} aria-label="Close" />
  </ModalHeader>

  <ModalBody class="space-y-5">
    {#if statusMessage}
      <div
        class="flex items-center gap-2 rounded-lg p-3 {statusMessage.type === 'success'
          ? 'bg-success/15 text-success'
          : 'bg-error/15 text-error'}"
      >
        <Icon icon={statusMessage.type === "success" ? "mdi:check-circle" : "mdi:alert-circle"} width="18" />
        <span class="text-sm">{statusMessage.text}</span>
      </div>
    {/if}

    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
        <Icon icon="mdi:database-export" width="18" class="text-accent" />
        Backup & Restore
      </h3>
      <p class="mb-4 text-sm text-foreground-muted">
        Export your current settings and tweak states to a backup file, or import from a previous backup.
      </p>

      <div class="flex flex-wrap gap-3">
        <Button variant="secondary" class="flex-1" onclick={handleExport} disabled={isExporting}>
          {#if isExporting}
            <Icon icon="mdi:loading" width="18" class="animate-spin" />
            Exporting...
          {:else}
            <Icon icon="mdi:export" width="18" />
            Export Backup
          {/if}
        </Button>

        <Button variant="secondary" class="flex-1" onclick={handleImport} disabled={isImporting}>
          {#if isImporting}
            <Icon icon="mdi:loading" width="18" class="animate-spin" />
            Importing...
          {:else}
            <Icon icon="mdi:import" width="18" />
            Import Backup
          {/if}
        </Button>
      </div>
    </div>

    <div class="flex items-start gap-3 rounded-lg border border-border/50 bg-surface/50 p-3">
      <Icon icon="mdi:information" width="18" class="mt-0.5 shrink-0 text-accent" />
      <p class="m-0 text-xs leading-relaxed text-foreground-muted">
        Backup files contain your app settings and a snapshot of all tweak states. When importing, settings will be
        restored immediately.
      </p>
    </div>
  </ModalBody>
</Modal>
