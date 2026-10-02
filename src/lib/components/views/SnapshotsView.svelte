<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { GroupedTweakList } from "$lib/components/tweaks";
  import { EmptyState, SkeletonCard } from "$lib/components/ui";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweakOps } from "$lib/stores/tweakOps.svelte";
  import { restoreTweaks } from "$lib/stores/tweaksActions.svelte";
  import { initStatus, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { canRestore, restoreMessage } from "$lib/utils/tweakPresentation";

  const snapshotTweaks = $derived(tweaksStore.list.filter((t) => t.status.hasSnapshot));
  const filteredTweaks = $derived(snapshotTweaks.filter((t) => pageFilterStore.passes(t.definition.id)));
  const restorable = $derived(snapshotTweaks.filter(canRestore));
  const appliedCount = $derived(snapshotTweaks.filter((t) => t.status.state === "active").length);

  async function restoreAll() {
    const ids = restorable.map((t) => t.definition.id);
    const ok = await confirmStore.ask({
      title: "Restore all snapshots?",
      message: restoreMessage(ids.length),
      confirmText: "Restore",
      variant: "danger",
    });
    if (ok) await restoreTweaks(ids);
  }
</script>

<PageLayout
  title="Snapshots"
  description="Tweaks with a saved snapshot. Each Restore steps a tweak back to the state saved before its last change."
>
  {#snippet aside()}
    {#if snapshotTweaks.length > 0}
      <div class="flex w-full flex-wrap items-center gap-x-4 gap-y-2">
        <p class="m-0 text-xs text-foreground-muted">
          <span class="font-semibold text-foreground tabular-nums">{snapshotTweaks.length}</span> with snapshots ·
          <span class="font-semibold text-foreground tabular-nums">{appliedCount}</span> currently applied
        </p>
        <button
          type="button"
          class="ml-auto inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-ui font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
          disabled={tweakOps.isBusy || restorable.length === 0}
          onclick={restoreAll}
        >
          <Icon icon="mdi:history" width="16" />
          Restore all
          <span class="text-xs text-foreground-subtle tabular-nums">{restorable.length}</span>
        </button>
      </div>
    {/if}
  {/snippet}

  {#if initStatus.isLoadingTweaks && snapshotTweaks.length === 0}
    <SkeletonCard />
  {:else if snapshotTweaks.length === 0}
    <EmptyState
      icon="mdi:backup-restore"
      title="No snapshots yet"
      description="Applying a tweak saves the state it replaces as a snapshot, so you can restore it later."
      actionText="Browse tweaks"
      onaction={() => navigationStore.navigateToTab("overview")}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <EmptyState
      icon="mdi:file-search-outline"
      title="Nothing matches"
      description={`No snapshots match "${pageFilterStore.query.trim()}"`}
      actionText="Search everywhere"
      onaction={() => pageFilterStore.searchEverywhere()}
    />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>
