<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { GroupedTweakList } from "$lib/components/tweaks";
  import { EmptyState, SearchInput, SkeletonCard } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { batchRevertTweaks, loadingStateStore, loadingStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { canRestore, matchesQuery, restoreMessage } from "$lib/utils/tweakPresentation";

  let searchQuery = $state("");
  let showRevertAllDialog = $state(false);

  const snapshotTweaks = $derived(tweaksStore.list.filter((t) => t.status.has_backup));
  const filteredTweaks = $derived(snapshotTweaks.filter((t) => matchesQuery(t, searchQuery)));
  const restorable = $derived(snapshotTweaks.filter(canRestore));
  const appliedCount = $derived(snapshotTweaks.filter((t) => t.status.is_applied).length);

  async function handleRestoreAll() {
    showRevertAllDialog = false;
    await batchRevertTweaks(restorable.map((t) => t.definition.id));
  }
</script>

<PageLayout
  title="Snapshots"
  description="Tweaks with a saved snapshot. Each Restore steps a tweak back to the state saved before its last change."
>
  {#snippet aside()}
    {#if snapshotTweaks.length > 0}
      <p class="m-0 text-xs text-foreground-muted">
        <span class="font-semibold text-foreground tabular-nums">{snapshotTweaks.length}</span> with snapshots ·
        <span class="font-semibold text-foreground tabular-nums">{appliedCount}</span> currently applied
      </p>
    {/if}
  {/snippet}

  {#snippet toolbar()}
    {#if snapshotTweaks.length > 0}
      <SearchInput
        value={searchQuery}
        placeholder="Filter snapshots"
        class="min-w-48 flex-1 sm:max-w-80"
        onchange={(v) => (searchQuery = v)}
      />
      <button
        type="button"
        class="ml-auto inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
        disabled={loadingStore.busy || restorable.length === 0}
        onclick={() => (showRevertAllDialog = true)}
      >
        <Icon icon="mdi:history" width="16" />
        Restore all
        <span class="text-xs text-foreground-subtle tabular-nums">{restorable.length}</span>
      </button>
    {/if}
  {/snippet}

  {#if loadingStateStore.tweaksLoading && snapshotTweaks.length === 0}
    <SkeletonCard />
  {:else if snapshotTweaks.length === 0}
    <EmptyState
      icon="mdi:backup-restore"
      title="No snapshots yet"
      description="Applying a tweak saves the state it replaces as a snapshot, so you can restore it later."
      actionText="Browse tweaks"
      onaction={() => navigationStore.navigateToOverview()}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <EmptyState
      icon="mdi:file-search-outline"
      title="Nothing matches"
      description={`No snapshots match "${searchQuery}"`}
      actionText="Clear filter"
      onaction={() => (searchQuery = "")}
    />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore all snapshots"
  message={restoreMessage(restorable.length)}
  confirmText="Restore"
  variant="danger"
  onconfirm={handleRestoreAll}
  oncancel={() => (showRevertAllDialog = false)}
/>
