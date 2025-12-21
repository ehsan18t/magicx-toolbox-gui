<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { TweakCard } from "$lib/components/tweaks";
  import { ActionButton, EmptyState, SkeletonCard } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import {
    applyPendingChanges,
    batchRevertTweaks,
    categoriesStore,
    loadingStateStore,
    loadingStore,
    pendingChangesStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";

  let searchQuery = $state("");
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Check if tweaks are still loading
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

  // Get tweaks that have snapshots (has_backup)
  const snapshotTweaks = $derived(tweaksStore.list.filter((t) => t.status.has_backup));

  // Filter tweaks by search
  const filteredTweaks = $derived.by(() => {
    if (!searchQuery.trim()) return snapshotTweaks;
    const query = searchQuery.toLowerCase();
    return snapshotTweaks.filter(
      (t) => t.definition.name.toLowerCase().includes(query) || t.definition.description.toLowerCase().includes(query),
    );
  });

  // Group tweaks by category for display
  const tweaksByCategory = $derived.by(() => {
    const groups: Record<string, typeof filteredTweaks> = {};

    for (const tweak of filteredTweaks) {
      const catId = tweak.definition.category_id;
      if (!groups[catId]) {
        groups[catId] = [];
      }
      groups[catId].push(tweak);
    }

    return groups;
  });

  // Stats
  const totalCount = $derived(snapshotTweaks.length);
  const appliedCount = $derived(snapshotTweaks.filter((t) => t.status.is_applied).length);

  // Pending changes for snapshot tweaks
  const pendingCount = $derived(pendingChangesStore.getCountForTweaks(snapshotTweaks.map((t) => t.definition.id)));

  // Loading state
  const isLoading = $derived(snapshotTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  async function handleApplyChanges() {
    showApplyAllDialog = false;
    isBatchProcessing = true;
    await applyPendingChanges();
    isBatchProcessing = false;
  }

  async function handleRestoreAll() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    await batchRevertTweaks(snapshotTweaks.map((t) => t.definition.id));

    isBatchProcessing = false;
  }

  function handleDiscardChanges() {
    // Clear pending changes for all snapshot tweaks
    const tweakIds = new Set(snapshotTweaks.map((t) => t.definition.id));
    for (const [tweakId] of pendingChangesStore.all) {
      if (tweakIds.has(tweakId)) {
        pendingChangesStore.clear(tweakId);
      }
    }
  }

  function navigateToCategory(categoryId: string) {
    navigationStore.navigateToCategory(categoryId);
  }
</script>

<div class="flex h-full flex-col gap-5 overflow-hidden p-6">
  <!-- Header -->
  <header class="flex flex-wrap items-center justify-between gap-6">
    <div class="flex items-center gap-4">
      <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-accent/15 text-accent">
        <Icon icon="mdi:backup-restore" width="28" />
      </div>
      <div>
        <h1 class="m-0 text-2xl font-bold tracking-tight text-foreground">Snapshots</h1>
        <p class="mt-1 mb-0 text-sm text-foreground-muted">Tweaks with saved original state that can be restored</p>
      </div>
    </div>

    <div class="flex items-center gap-4 rounded-xl border border-border bg-card px-5 py-3">
      <div class="flex items-center gap-2.5">
        <div class="flex h-9 w-9 items-center justify-center rounded-full bg-accent/15">
          <Icon icon="mdi:history" width="18" class="text-accent" />
        </div>
        <div class="flex flex-col items-center justify-center gap-0.5">
          <span class="text-base font-bold text-foreground">{totalCount}</span>
          <span class="text-xs text-foreground-muted">Snapshots</span>
        </div>
      </div>
      <div class="h-8 w-px bg-border"></div>
      <div class="flex flex-col items-center justify-center gap-0.5">
        <span class="text-base font-bold text-foreground">{appliedCount}</span>
        <span class="text-xs text-foreground-muted">Currently Applied</span>
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
        placeholder="Search snapshots..."
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
        active={pendingCount > 0}
        loading={isBatchProcessing}
        badgeCount={pendingCount}
        badgeVariant="warning"
        onclick={() => (showApplyAllDialog = true)}
        disabled={pendingCount === 0 || isLoading || isBatchProcessing}
      >
        Apply Changes
      </ActionButton>
      <ActionButton
        intent="discard"
        icon="mdi:close-circle-outline"
        onclick={handleDiscardChanges}
        disabled={pendingCount === 0 || isLoading || isBatchProcessing}
        tooltip="Discard all pending changes"
      >
        Discard
      </ActionButton>
      <ActionButton
        intent="restore"
        icon="mdi:restore"
        badgeCount={totalCount}
        badgeVariant="error"
        onclick={() => (showRevertAllDialog = true)}
        disabled={totalCount === 0 || isLoading || isBatchProcessing}
        tooltip={totalCount === 0 ? "No snapshots available" : `Restore all ${totalCount} snapshots`}
      >
        Restore All
      </ActionButton>
    </div>
  </div>

  <!-- Tweaks Grid - grouped by category -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if tweaksLoading && snapshotTweaks.length === 0}
      <SkeletonCard />
    {:else if snapshotTweaks.length === 0}
      <EmptyState
        icon="mdi:backup-restore"
        title="No Snapshots Yet"
        description="When you apply tweaks, their original state is saved as a snapshot. You can restore these snapshots later to undo changes."
        actionText="Browse Tweaks"
        onaction={() => navigationStore.navigateToOverview()}
        showIconCircle
      />
    {:else if filteredTweaks.length === 0}
      <EmptyState
        icon="mdi:file-search-outline"
        title="No results found"
        description={`No snapshots match "${searchQuery}"`}
        actionText="Clear search"
        onaction={() => (searchQuery = "")}
      />
    {:else}
      <!-- Grouped by category -->
      <div class="flex flex-col gap-6 pb-4">
        {#each Object.entries(tweaksByCategory) as [categoryId, tweaks] (categoryId)}
          <div class="flex flex-col gap-3">
            <!-- Category Header -->
            <button
              type="button"
              class="hover:bg-muted/50 flex cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent px-1 py-1 transition-all duration-150"
              onclick={() => navigateToCategory(categoryId)}
              use:tooltip={`View ${categoriesStore.getName(categoryId)} category`}
            >
              <Icon icon={categoriesStore.getIcon(categoryId)} width="18" class="text-accent" />
              <h3 class="m-0 text-sm font-semibold text-foreground">{categoriesStore.getName(categoryId)}</h3>
              <span class="text-xs text-foreground-muted">({tweaks.length})</span>
              <Icon icon="mdi:chevron-right" width="16" class="text-foreground-muted" />
            </button>

            <!-- Tweaks Grid -->
            <div class="flex flex-col gap-3 lg:grid lg:grid-cols-2 lg:gap-4">
              {#each tweaks as tweak (tweak.definition.id)}
                <TweakCard {tweak} />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Dialogs -->
<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply Pending Changes"
  message="Apply {pendingCount} pending change(s)? Some tweaks may require a system restart."
  confirmText="Apply Changes"
  variant="default"
  onconfirm={handleApplyChanges}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore All Snapshots"
  message="Restore all {totalCount} tweak{totalCount > 1
    ? 's'
    : ''} to their original state? This will undo all applied changes."
  confirmText="Restore All"
  variant="danger"
  onconfirm={handleRestoreAll}
  oncancel={() => (showRevertAllDialog = false)}
/>
