<script lang="ts">
  import { IconButton, type IconButtonSize } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";

  interface Props {
    tweakId: string;
    /** The tweak's name, so each row's toggle is told apart. */
    name: string;
    size?: IconButtonSize;
  }

  let { tweakId, name, size = "sm" }: Props = $props();

  const isFavorite = $derived(favoritesStore.isFavorite(tweakId));
</script>

<IconButton
  {size}
  icon={isFavorite ? "mdi:star" : "mdi:star-outline"}
  label="Favorite {name}"
  tooltip={isFavorite ? "Remove from favorites" : "Add to favorites"}
  aria-pressed={isFavorite}
  active={isFavorite}
  tone="warning"
  onclick={() => favoritesStore.toggle(tweakId)}
/>
