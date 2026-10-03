<script lang="ts">
  import { NoMatches, PageLayout, PageStats } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { GroupedTweakList, RestoreAllButton } from "$lib/components/tweaks";
  import { Button, EmptyState, ICON_SIZE, SkeletonList } from "$lib/components/ui";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { plural } from "$lib/utils/format";
  import { canRestore, tallies } from "$lib/utils/tweakPresentation";

  const favorites = $derived(tweaksStore.favorites);
  const filteredTweaks = $derived(favorites.filter((t) => pageFilterStore.passes(t.definition.id)));
  const restorable = $derived(favorites.filter(canRestore));
  const applied = $derived(tallies(favorites).applied);

  async function clearAll() {
    const ok = await confirmStore.ask({
      title: "Clear all favorites?",
      message: `Remove ${plural(favorites.length, "tweak")} from your favorites? This won't change the tweaks themselves.`,
      confirmText: "Clear favorites",
      variant: "danger",
    });
    if (!ok) return;
    favoritesStore.clear();
    toastStore.success("All favorites cleared");
  }
</script>

<PageLayout title="Favorites" description="Quick access to the tweaks you starred.">
  {#snippet aside()}
    {#if favorites.length > 0}
      <div class="flex w-full flex-wrap items-center gap-x-4 gap-y-2">
        <PageStats
          items={[
            { value: favorites.length, label: "starred" },
            { value: applied, label: "applied" },
          ]}
        />
        <div class="ml-auto flex flex-wrap gap-2">
          {#if restorable.length > 0}
            <RestoreAllButton title="Restore favorites?" tweaks={restorable} />
          {/if}
          <Button variant="ghost" disabled={tweakActionsStore.isBusy} onclick={clearAll}>
            <Icon icon="mdi:star-off" width={ICON_SIZE.md} />
            Clear favorites
          </Button>
        </div>
      </div>
    {/if}
  {/snippet}

  {#if tweaksStore.isLoading && favorites.length === 0}
    <SkeletonList />
  {:else if favorites.length === 0}
    <EmptyState
      icon="mdi:star-outline"
      title="No favorites yet"
      description="Select the star on any tweak to keep it here for quick access."
      action={{ label: "Browse tweaks", onclick: () => navigationStore.navigateToTab("overview") }}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <NoMatches description={`No favorites match "${pageFilterStore.query.trim()}"`} />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>
