<script lang="ts">
  import { IconButton } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";

  interface Props {
    tweakId: string;
    size: "sm" | "md";
  }

  let { tweakId, size }: Props = $props();

  const isFavorite = $derived(favoritesStore.isFavorite(tweakId));
</script>

<IconButton
  {size}
  icon={isFavorite ? "mdi:star" : "mdi:star-outline"}
  tooltip={isFavorite ? "Remove from favorites" : "Add to favorites"}
  aria-pressed={isFavorite}
  class={isFavorite ? "text-warning enabled:hover:text-warning" : undefined}
  onclick={() => favoritesStore.toggle(tweakId)}
/>
