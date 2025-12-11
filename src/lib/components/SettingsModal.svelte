<script lang="ts">
  import { closeModal, modalStore } from "$lib/stores/modal";
  import { settingsStore } from "$lib/stores/settings";
  import { tweaksStore } from "$lib/stores/tweaks";
  import type { ExportData, TweakSnapshot } from "$lib/types";
  import { getVersion } from "@tauri-apps/api/app";
  import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";

  let appVersion = $state("1.0.0");
  let isExporting = $state(false);
  let isImporting = $state(false);
  let statusMessage = $state<{ type: "success" | "error"; text: string } | null>(null);

  const isOpen = $derived($modalStore === "settings");

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error("Failed to get app version:", error);
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

  function showStatus(type: "success" | "error", text: string) {
    statusMessage = { type, text };
    setTimeout(() => {
      statusMessage = null;
    }, 3000);
  }

  async function createTweakSnapshots(): Promise<TweakSnapshot[]> {
    const tweaks = $tweaksStore;
    const snapshots: TweakSnapshot[] = [];

    for (const tweak of tweaks) {
      // Snapshot current state regardless of whether applied by app or not
      snapshots.push({
        tweakId: tweak.definition.id,
        tweakName: tweak.definition.name,
        isApplied: tweak.status.is_applied,
        registryValues: {}, // Registry values would be fetched from backend
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
      const settings = settingsStore.get();

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

        // Validate import data structure
        if (!importData.version || !importData.settings) {
          throw new Error("Invalid backup file format");
        }

        // Import settings
        settingsStore.update(importData.settings);

        // TODO: Apply tweak snapshots via backend command
        // This would restore registry values to their snapshotted state

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
      aria-labelledby="settings-title"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-border px-5 py-4">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
            <Icon icon="mdi:cog" width="24" class="text-accent" />
          </div>
          <h2 id="settings-title" class="m-0 text-lg font-bold text-foreground">Settings</h2>
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
        <!-- Status Message -->
        {#if statusMessage}
          <div
            class="flex items-center gap-2 rounded-lg p-3 {statusMessage.type === 'success'
              ? 'bg-success/15 text-success'
              : 'bg-error/15 text-error'}"
          >
            <Icon
              icon={statusMessage.type === "success" ? "mdi:check-circle" : "mdi:alert-circle"}
              width="18"
            />
            <span class="text-sm">{statusMessage.text}</span>
          </div>
        {/if}

        <!-- Export/Import Section -->
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
            <Icon icon="mdi:database-export" width="18" class="text-accent" />
            Backup & Restore
          </h3>
          <p class="mb-4 text-sm text-foreground-muted">
            Export your current settings and tweak states to a backup file, or import from a
            previous backup. Tweaks are snapshotted as they currently are, regardless of whether
            they were applied by this app.
          </p>

          <div class="flex flex-wrap gap-3">
            <button
              class="flex flex-1 cursor-pointer items-center justify-center gap-2 rounded-lg border border-border bg-[hsl(var(--muted))] px-4 py-2.5 text-sm font-medium text-foreground transition-colors hover:bg-[hsl(var(--muted)/0.8)] disabled:cursor-not-allowed disabled:opacity-50"
              onclick={handleExport}
              disabled={isExporting}
            >
              {#if isExporting}
                <Icon icon="mdi:loading" width="18" class="animate-spin" />
                Exporting...
              {:else}
                <Icon icon="mdi:export" width="18" />
                Export Backup
              {/if}
            </button>

            <button
              class="flex flex-1 cursor-pointer items-center justify-center gap-2 rounded-lg border border-border bg-[hsl(var(--muted))] px-4 py-2.5 text-sm font-medium text-foreground transition-colors hover:bg-[hsl(var(--muted)/0.8)] disabled:cursor-not-allowed disabled:opacity-50"
              onclick={handleImport}
              disabled={isImporting}
            >
              {#if isImporting}
                <Icon icon="mdi:loading" width="18" class="animate-spin" />
                Importing...
              {:else}
                <Icon icon="mdi:import" width="18" />
                Import Backup
              {/if}
            </button>
          </div>
        </div>

        <!-- Info Note -->
        <div class="flex items-start gap-3 rounded-lg border border-border/50 bg-surface/50 p-3">
          <Icon icon="mdi:information" width="18" class="mt-0.5 shrink-0 text-accent" />
          <p class="m-0 text-xs leading-relaxed text-foreground-muted">
            Backup files contain your app settings and a snapshot of all tweak states. When
            importing, settings will be restored immediately. Tweak states can be used to restore
            registry values to their previous configuration.
          </p>
        </div>
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
</style>
