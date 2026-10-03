<script lang="ts">
  import { Button } from "$lib/components/ui";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { plural } from "$lib/utils/format";
  import TweakCollectionView from "./TweakCollectionView.svelte";

  const favorites = $derived(tweaksStore.favorites);

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
    <Button variant="ghost" icon="mdi:star-off" disabled={tweakActionsStore.isBusy} onclick={clearAll}>
      Clear favorites
    </Button>
  {/snippet}
</TweakCollectionView>
