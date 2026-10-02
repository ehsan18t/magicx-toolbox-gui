<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { GroupedTweakList } from "$lib/components/tweaks";
  import { EmptyState, SearchInput, SkeletonCard } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { batchRevertTweaks, loadingStateStore, loadingStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { matchesQuery } from "$lib/utils/tweakPresentation";

  let searchQuery = $state("");
  let showRevertAllDialog = $state(false);
  let showClearAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  const favoriteTweaks = $derived(tweaksStore.list.filter((t) => favoritesStore.ids.includes(t.definition.id)));
  const filteredTweaks = $derived(favoriteTweaks.filter((t) => matchesQuery(t, searchQuery)));
  const withSnapshots = $derived(favoriteTweaks.filter((t) => t.status.has_backup));
  const appliedCount = $derived(favoriteTweaks.filter((t) => t.status.is_applied).length);
  const isLoading = $derived(favoriteTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  async function handleRestoreAll() {
    showRevertAllDialog = false;
    isBatchProcessing = true;
    await batchRevertTweaks(withSnapshots.map((t) => t.definition.id));
    isBatchProcessing = false;
  }

  function handleClearAll() {
    showClearAllDialog = false;
    favoritesStore.clear();
    toastStore.success("All favorites cleared");
  }
</script>

<PageLayout title="Favorites" description="Quick access to the tweaks you starred.">
  {#snippet aside()}
    {#if favoriteTweaks.length > 0}
      <p class="m-0 text-xs text-foreground-muted">
        <span class="font-semibold text-foreground tabular-nums">{favoriteTweaks.length}</span> starred ·
        <span class="font-semibold text-foreground tabular-nums">{appliedCount}</span> applied
      </p>
    {/if}
  {/snippet}

  {#snippet toolbar()}
    {#if favoriteTweaks.length > 0}
      <SearchInput
        value={searchQuery}
        placeholder="Filter favorites"
        class="min-w-48 flex-1 sm:max-w-80"
        onchange={(v) => (searchQuery = v)}
      />
      <div class="ml-auto flex flex-wrap gap-2">
        {#if withSnapshots.length > 0}
          <button
            type="button"
            class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isLoading || isBatchProcessing}
            onclick={() => (showRevertAllDialog = true)}
          >
            <Icon icon="mdi:history" width="16" />
            Restore all
            <span class="text-xs text-foreground-subtle tabular-nums">{withSnapshots.length}</span>
          </button>
        {/if}
        <button
          type="button"
          class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md px-3 text-[13px] font-medium text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isBatchProcessing}
          onclick={() => (showClearAllDialog = true)}
        >
          <Icon icon="mdi:star-off" width="16" />
          Clear favorites
        </button>
      </div>
    {/if}
  {/snippet}

  {#if loadingStateStore.tweaksLoading && favoriteTweaks.length === 0}
    <SkeletonCard />
  {:else if favoriteTweaks.length === 0}
    <EmptyState
      icon="mdi:star-outline"
      title="No favorites yet"
      description="Select the star on any tweak to keep it here for quick access."
      actionText="Browse tweaks"
      onaction={() => navigationStore.navigateToOverview()}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <EmptyState
      icon="mdi:file-search-outline"
      title="Nothing matches"
      description={`No favorites match "${searchQuery}"`}
      actionText="Clear filter"
      onaction={() => (searchQuery = "")}
    />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore All Snapshots"
  message="Restore {withSnapshots.length} favorite{withSnapshots.length === 1
    ? ''
    : 's'} to their original state from saved snapshots?"
  confirmText="Restore All"
  variant="danger"
  onconfirm={handleRestoreAll}
  oncancel={() => (showRevertAllDialog = false)}
/>

<ConfirmDialog
  open={showClearAllDialog}
  title="Clear All Favorites"
  message="Remove all {favoriteTweaks.length} tweak{favoriteTweaks.length === 1
    ? ''
    : 's'} from your favorites? This won't change the tweaks themselves."
  confirmText="Clear Favorites"
  variant="danger"
  onconfirm={handleClearAll}
  oncancel={() => (showClearAllDialog = false)}
/>
