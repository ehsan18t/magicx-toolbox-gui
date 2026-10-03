<script lang="ts">
  import { Button } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { pageTweaks } from "$lib/stores/navigation.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import TweakCollectionView from "./TweakCollectionView.svelte";

  const favorites = $derived(pageTweaks("favorites") ?? []);
</script>

<TweakCollectionView
  title="Favorites"
  description="Quick access to the tweaks you starred."
  tweaks={favorites}
  noun="favorites"
  countLabel="starred"
  appliedLabel="applied"
  restoreTitle="Restore favorites?"
  empty={{
    icon: "mdi:star-outline",
    title: "No favorites yet",
    description: "Select the star on any tweak to keep it here for quick access.",
  }}
>
  {#snippet actions()}
    <Button
      variant="ghost"
      icon="mdi:star-off"
      disabled={tweakActionsStore.isBusy}
      onclick={() => favoritesStore.clearWithConfirm(favorites.map((t) => t.definition.id))}
    >
      Clear favorites
    </Button>
  {/snippet}
</TweakCollectionView>
