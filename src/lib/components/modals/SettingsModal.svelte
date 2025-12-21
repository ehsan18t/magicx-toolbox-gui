<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, IconButton, Modal, ModalBody, ModalHeader } from "$lib/components/ui";
  import { closeModal, modalStore, openProfileExportModal, openProfileImportModal } from "$lib/stores/modal.svelte";
  import { tweaksStore } from "$lib/stores/tweaks.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";

  let appVersion = $state("1.0.0");

  const isOpen = $derived(modalStore.current === "settings");

  // Count applied tweaks for badge
  const appliedCount = $derived(tweaksStore.list.filter((t) => t.status.is_applied).length);

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  });

  function handleExportProfile() {
    closeModal();
    // Small delay to allow current modal to close
    setTimeout(() => {
      openProfileExportModal();
    }, 100);
  }

  function handleImportProfile() {
    closeModal();
    setTimeout(() => {
      openProfileImportModal();
    }, 100);
  }
</script>

<Modal open={isOpen} onclose={closeModal} size="md" labelledBy="settings-modal-title">
  <ModalHeader id="settings-modal-title">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
        <Icon icon="mdi:cog" width="24" class="text-accent" />
      </div>
      <h2 class="m-0 text-lg font-bold text-foreground">Settings</h2>
    </div>
    <IconButton icon="mdi:close" onclick={closeModal} aria-label="Close" />
  </ModalHeader>

  <ModalBody class="space-y-5">
    <!-- Configuration Profiles Section -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
        <Icon icon="mdi:file-document-multiple" width="18" class="text-accent" />
        Configuration Profiles
      </h3>
      <p class="mb-4 text-sm text-foreground-muted">
        Export your tweak configurations to share across machines or back up before reinstalling Windows.
      </p>

      <div class="flex flex-wrap gap-3">
        <Button variant="secondary" class="flex-1" onclick={handleExportProfile}>
          <Icon icon="mdi:export" width="18" />
          Export Profile
          {#if appliedCount > 0}
            <Badge variant="default" class="ml-1">{appliedCount}</Badge>
          {/if}
        </Button>

        <Button variant="secondary" class="flex-1" onclick={handleImportProfile}>
          <Icon icon="mdi:import" width="18" />
          Import Profile
        </Button>
      </div>
    </div>

    <!-- Profile Info Box -->
    <div class="flex items-start gap-3 rounded-lg border border-border/50 bg-surface/50 p-3">
      <Icon icon="mdi:information" width="18" class="mt-0.5 shrink-0 text-accent" />
      <div class="text-xs leading-relaxed text-foreground-muted">
        <p class="m-0">
          <strong>Profiles</strong> save your applied tweak selections as a portable
          <code class="bg-muted rounded px-1">.mgx</code> file.
        </p>
        <ul class="m-0 mt-1.5 list-inside list-disc space-y-0.5 pl-0">
          <li>Share your setup with others</li>
          <li>Restore after Windows reinstall</li>
          <li>Sync across multiple machines</li>
        </ul>
      </div>
    </div>

    <!-- App Info -->
    <div class="flex items-center justify-between rounded-lg border border-border/50 bg-surface/50 px-4 py-3">
      <span class="text-sm text-foreground-muted">App Version</span>
      <Badge variant="default">{appVersion}</Badge>
    </div>
  </ModalBody>
</Modal>
