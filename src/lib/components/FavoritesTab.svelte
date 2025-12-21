<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ActionButton, EmptyState, SkeletonCard } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import {
    applyPendingChanges,
    batchRevertTweaks,
    categoriesStore,
    loadingStateStore,
    loadingStore,
    pendingChangesStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import TweakCard from "./TweakCard.svelte";

  let searchQuery = $state("");
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let showClearAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Check if tweaks are still loading
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

  // Get favorited tweaks from tweaksStore
  const favoriteTweaks = $derived.by(() => {
    const ids = favoritesStore.ids;
    return tweaksStore.list.filter((t) => ids.includes(t.definition.id));
  });

  // Filter tweaks by search
  const filteredTweaks = $derived.by(() => {
    if (!searchQuery.trim()) return favoriteTweaks;
    const query = searchQuery.toLowerCase();
    return favoriteTweaks.filter(
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
  const totalCount = $derived(favoriteTweaks.length);
  const appliedCount = $derived(favoriteTweaks.filter((t) => t.status.is_applied).length);
  const snapshotCount = $derived(favoriteTweaks.filter((t) => t.status.has_backup).length);

  // Pending changes for favorite tweaks
  const pendingCount = $derived(pendingChangesStore.getCountForTweaks(favoriteTweaks.map((t) => t.definition.id)));

  // Loading state
  const isLoading = $derived(favoriteTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  async function handleApplyChanges() {
    showApplyAllDialog = false;
    isBatchProcessing = true;
    await applyPendingChanges();
    isBatchProcessing = false;
  }

  async function handleRestoreAll() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    const tweaksWithSnapshots = favoriteTweaks.filter((t) => t.status.has_backup);
    await batchRevertTweaks(tweaksWithSnapshots.map((t) => t.definition.id));

    isBatchProcessing = false;
  }

  function handleDiscardChanges() {
    // Clear pending changes for all favorite tweaks
    const tweakIds = new Set(favoriteTweaks.map((t) => t.definition.id));
    for (const [tweakId] of pendingChangesStore.all) {
      if (tweakIds.has(tweakId)) {
        pendingChangesStore.clear(tweakId);
      }
    }
  }

  function handleClearAllFavorites() {
    showClearAllDialog = false;
    favoritesStore.clear();
    toastStore.success("All favorites cleared");
  }

  function navigateToCategory(categoryId: string) {
    navigationStore.navigateToCategory(categoryId);
  }
</script>

<div class="flex h-full flex-col gap-5 overflow-hidden p-6">
  <!-- Header -->
  <header class="flex flex-wrap items-center justify-between gap-6">
    <div class="flex items-center gap-4">
      <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-warning/15 text-warning">
        <Icon icon="mdi:star" width="28" />
      </div>
      <div>
        <h1 class="m-0 text-2xl font-bold tracking-tight text-foreground">Favorites</h1>
        <p class="mt-1 mb-0 text-sm text-foreground-muted">Quick access to your saved tweaks</p>
      </div>
    </div>

    <div class="flex items-center gap-4 rounded-xl border border-border bg-card px-5 py-3">
      <div class="flex items-center gap-2.5">
        <div class="flex h-9 w-9 items-center justify-center rounded-full bg-warning/15">
          <Icon icon="mdi:star" width="18" class="text-warning" />
        </div>
        <div class="flex flex-col gap-0.5">
          <span class="text-base font-bold text-foreground">{totalCount}</span>
          <span class="text-xs text-foreground-muted">Favorites</span>
        </div>
      </div>
      <div class="h-8 w-px bg-border"></div>
      <div class="flex flex-col gap-0.5">
        <span class="text-base font-bold text-foreground">{appliedCount}</span>
        <span class="text-xs text-foreground-muted">Applied</span>
      </div>
      {#if snapshotCount > 0}
        <div class="h-8 w-px bg-border"></div>
        <div class="flex flex-col gap-0.5">
          <span class="text-base font-bold text-foreground">{snapshotCount}</span>
          <span class="text-xs text-foreground-muted">Snapshots</span>
        </div>
      {/if}
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
        placeholder="Search favorites..."
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
        intent="accent"
        icon="mdi:restore"
        badgeCount={snapshotCount}
        badgeVariant="accent"
        onclick={() => (showRevertAllDialog = true)}
        disabled={snapshotCount === 0 || isLoading || isBatchProcessing}
        tooltip={snapshotCount === 0 ? "No snapshots to restore" : `Restore all ${snapshotCount} snapshots`}
      >
        Restore All
      </ActionButton>
      <ActionButton
        intent="danger"
        icon="mdi:star-off"
        onclick={() => (showClearAllDialog = true)}
        disabled={totalCount === 0 || isBatchProcessing}
        tooltip={totalCount === 0 ? "No favorites to clear" : "Clear all favorites"}
      >
        Clear All
      </ActionButton>
    </div>
  </div>

  <!-- Tweaks Grid - grouped by category -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if tweaksLoading && favoriteTweaks.length === 0}
      <SkeletonCard />
    {:else if favoriteTweaks.length === 0}
      <EmptyState
        icon="mdi:star-outline"
        title="No Favorites Yet"
        description="Click the star icon on any tweak to add it to your favorites for quick access."
        actionText="Browse Tweaks"
        onaction={() => navigationStore.navigateToOverview()}
        showIconCircle
      />
    {:else if filteredTweaks.length === 0}
      <EmptyState
        icon="mdi:file-search-outline"
        title="No results found"
        description={`No favorites match "${searchQuery}"`}
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
  message="Restore {snapshotCount} tweak{snapshotCount > 1
    ? 's'
    : ''} to their original state? This will undo all applied changes for favorites with snapshots."
  confirmText="Restore All"
  variant="danger"
  onconfirm={handleRestoreAll}
  oncancel={() => (showRevertAllDialog = false)}
/>

<ConfirmDialog
  open={showClearAllDialog}
  title="Clear All Favorites"
  message="Remove all {totalCount} tweak{totalCount > 1
    ? 's'
    : ''} from your favorites? This won't affect the tweaks themselves."
  confirmText="Clear Favorites"
  variant="danger"
  onconfirm={handleClearAllFavorites}
  oncancel={() => (showClearAllDialog = false)}
/>
