<script lang="ts">
  import { ActionButton, EmptyState, SkeletonCard } from "$lib/components/ui";
  import type { TabDefinition } from "$lib/stores/navigation.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import {
    applyPendingChanges,
    loadingStateStore,
    loadingStore,
    pendingChangesStore,
    revertTweak,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import TweakCard from "./TweakCard.svelte";

  interface Props {
    tab: TabDefinition;
  }

  const { tab }: Props = $props();

  let searchQuery = $state("");
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Check if tweaks are still loading
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

  // Get tweaks for this category
  const categoryTweaks = $derived(tweaksStore.list.filter((t) => t.definition.category_id === tab.id));

  // Filter tweaks by search
  const filteredTweaks = $derived.by(() => {
    if (!searchQuery.trim()) return categoryTweaks;
    const query = searchQuery.toLowerCase();
    return categoryTweaks.filter(
      (t) => t.definition.name.toLowerCase().includes(query) || t.definition.description.toLowerCase().includes(query),
    );
  });

  // Stats
  const appliedCount = $derived(categoryTweaks.filter((t) => t.status.is_applied).length);
  const totalCount = $derived(categoryTweaks.length);
  const progressPercent = $derived(totalCount > 0 ? Math.round((appliedCount / totalCount) * 100) : 0);

  // Tweaks with snapshots (can be restored)
  const tweaksWithSnapshots = $derived(categoryTweaks.filter((t) => t.status.has_backup));
  const snapshotCount = $derived(tweaksWithSnapshots.length);

  // Pending changes for this category
  const categoryPendingCount = $derived.by(() => {
    let count = 0;
    const pending = pendingChangesStore.all;
    const tweaks = tweaksStore.list;
    for (const [tweakId] of pending) {
      const tweak = tweaks.find((t) => t.definition.id === tweakId);
      if (tweak?.definition.category_id === tab.id) {
        count++;
      }
    }
    return count;
  });

  // Loading state
  const isLoading = $derived(categoryTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  async function handleApplyChanges() {
    showApplyAllDialog = false;
    isBatchProcessing = true;
    await applyPendingChanges();
    isBatchProcessing = false;
  }

  async function handleRestoreSnapshots() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    let success = 0;
    let failed = 0;

    for (const tweak of tweaksWithSnapshots) {
      // Pass { showToast: false } to suppress individual notifications
      const result = await revertTweak(tweak.definition.id, { showToast: false });
      if (result) {
        success++;
      } else {
        failed++;
      }
    }

    // Show summary toast
    if (failed === 0 && success > 0) {
      toastStore.success(`Restored ${success} snapshot${success > 1 ? "s" : ""} successfully`);
    } else if (failed > 0 && success > 0) {
      toastStore.warning(`Restored ${success}, failed ${failed} snapshot${failed > 1 ? "s" : ""}`);
    } else if (failed > 0) {
      toastStore.error(`Failed to restore ${failed} snapshot${failed > 1 ? "s" : ""}`);
    }

    isBatchProcessing = false;
  }

  function handleRestoreClick() {
    if (snapshotCount === 0) {
      toastStore.info("No snapshots available to restore in this category");
      return;
    }
    showRevertAllDialog = true;
  }

  function handleDiscardChanges() {
    pendingChangesStore.clearCategory(tab.id, tweaksStore.list);
  }
</script>

<div class="flex h-full flex-col gap-5 overflow-hidden p-6">
  <!-- Header -->
  <header class="flex flex-wrap items-center justify-between gap-6">
    <div class="flex items-center gap-4">
      <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-accent/15 text-accent">
        <Icon icon={tab.icon || "mdi:folder"} width="28" />
      </div>
      <div>
        <h1 class="m-0 text-2xl font-bold tracking-tight text-foreground">{tab.name}</h1>
        <p class="mt-1 mb-0 text-sm text-foreground-muted">{tab.description}</p>
      </div>
    </div>

    <div class="flex items-center gap-3 rounded-xl border border-border bg-card px-5 py-3">
      <div class="stat-ring relative h-10 w-10" style="--progress: {progressPercent}">
        <svg viewBox="0 0 36 36" class="h-full w-full -rotate-90">
          <circle class="fill-none stroke-[hsl(var(--muted))] stroke-3" cx="18" cy="18" r="14" />
          <circle class="stat-progress stroke-round fill-none stroke-accent stroke-3" cx="18" cy="18" r="14" />
        </svg>
        <span class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-[10px] font-bold text-foreground"
          >{progressPercent}%</span
        >
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-base font-bold text-foreground">{appliedCount} / {totalCount}</span>
        <span class="text-xs text-foreground-muted">Applied</span>
      </div>
    </div>
  </header>

  <!-- Toolbar -->
  <div class="flex flex-wrap items-center gap-3">
    <div
      class="flex max-w-full min-w-60 flex-1 items-center gap-2.5 rounded-lg border border-border bg-surface px-4 py-2.5 transition-all duration-200 focus-within:border-accent focus-within:bg-card"
    >
      <Icon icon="mdi:magnify" width="20" class="shrink-0 text-foreground-muted" />
      <input
        type="text"
        placeholder="Search tweaks..."
        bind:value={searchQuery}
        class="flex-1 border-0 bg-transparent text-sm text-foreground outline-none placeholder:text-foreground-subtle"
      />
      {#if searchQuery}
        <button
          type="button"
          class="hover:bg-muted flex cursor-pointer items-center justify-center rounded border-0 bg-transparent p-1 text-foreground-muted transition-all duration-150 hover:text-foreground"
          onclick={() => (searchQuery = "")}
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      {/if}
    </div>

    <div class="flex gap-2.5">
      <ActionButton
        intent="apply"
        icon="mdi:check-all"
        active={categoryPendingCount > 0}
        loading={isBatchProcessing}
        badgeCount={categoryPendingCount}
        badgeVariant="warning"
        onclick={() => (showApplyAllDialog = true)}
        disabled={categoryPendingCount === 0 || isLoading || isBatchProcessing}
      >
        Apply Changes
      </ActionButton>
      <ActionButton
        intent="discard"
        icon="mdi:close-circle-outline"
        onclick={handleDiscardChanges}
        disabled={categoryPendingCount === 0 || isLoading || isBatchProcessing}
        tooltip="Discard all pending changes in this category"
      >
        Discard
      </ActionButton>
      <ActionButton
        intent="restore"
        icon="mdi:restore"
        badgeCount={snapshotCount}
        badgeVariant="error"
        onclick={handleRestoreClick}
        disabled={snapshotCount === 0 || isLoading || isBatchProcessing}
        tooltip={snapshotCount === 0
          ? "No snapshots available"
          : `Restore ${snapshotCount} snapshot${snapshotCount > 1 ? "s" : ""}`}
      >
        Restore Snapshots
      </ActionButton>
    </div>
  </div>

  <!-- Tweaks Grid -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if tweaksLoading && categoryTweaks.length === 0}
      <SkeletonCard />
    {:else if filteredTweaks.length === 0}
      {#if searchQuery}
        <EmptyState
          icon="mdi:file-search-outline"
          title="No results found"
          description={`No tweaks match "${searchQuery}"`}
          actionText="Clear search"
          onaction={() => (searchQuery = "")}
        />
      {:else}
        <EmptyState
          icon="mdi:package-variant"
          title="No tweaks available"
          description="This category has no tweaks for your system"
        />
      {/if}
    {:else}
      <div class="flex flex-col gap-3 pb-4 lg:grid lg:grid-cols-2 lg:gap-4">
        {#each filteredTweaks as tweak (tweak.definition.id)}
          <TweakCard {tweak} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Dialogs -->
<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply Pending Changes"
  message="Apply {categoryPendingCount} pending change(s)? Some tweaks may require a system restart."
  confirmText="Apply Changes"
  variant="default"
  onconfirm={handleApplyChanges}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore Snapshots"
  message="Restore {snapshotCount} tweak{snapshotCount > 1 ? 's' : ''} to their original state from saved snapshots?"
  confirmText="Restore Snapshots"
  variant="danger"
  onconfirm={handleRestoreSnapshots}
  oncancel={() => (showRevertAllDialog = false)}
/>

<style>
  .stat-progress {
    stroke-dasharray: 88;
    stroke-dashoffset: calc(88 * (1 - var(--progress) / 100));
    transition: stroke-dashoffset 0.4s ease;
    stroke-linecap: round;
  }
</style>
