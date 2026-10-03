<script lang="ts">
  import { NoMatches, PageLayout, PageStats } from "$lib/components/layout";
  import { GroupedTweakList, RestoreAllButton } from "$lib/components/tweaks";
  import { EmptyState, SkeletonList } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { canRestore, tallies } from "$lib/utils/tweakPresentation";

  const snapshotTweaks = $derived(tweaksStore.withSnapshot);
  const filteredTweaks = $derived(snapshotTweaks.filter((t) => pageFilterStore.passes(t.definition.id)));
  const restorable = $derived(snapshotTweaks.filter(canRestore));
  const applied = $derived(tallies(snapshotTweaks).applied);
</script>

<PageLayout
  title="Snapshots"
  description="Tweaks with a saved snapshot. Each Restore steps a tweak back to the state saved before its last change."
>
  {#snippet aside()}
    {#if snapshotTweaks.length > 0}
      <div class="flex w-full flex-wrap items-center gap-x-4 gap-y-2">
        <PageStats
          items={[
            { value: snapshotTweaks.length, label: "with snapshots" },
            { value: applied, label: "currently applied" },
          ]}
        />
        <RestoreAllButton title="Restore all snapshots?" tweaks={restorable} class="ml-auto" />
      </div>
    {/if}
  {/snippet}

  {#if tweaksStore.isLoading && snapshotTweaks.length === 0}
    <SkeletonList />
  {:else if snapshotTweaks.length === 0}
    <EmptyState
      icon="mdi:backup-restore"
      title="No snapshots yet"
      description="Applying a tweak saves the state it replaces as a snapshot, so you can restore it later."
      action={{ label: "Browse tweaks", onclick: () => navigationStore.navigateToTab("overview") }}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <NoMatches description={`No snapshots match "${pageFilterStore.query.trim()}"`} />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>
