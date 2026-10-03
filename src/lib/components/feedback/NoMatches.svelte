<script lang="ts">
  import { EmptyState } from "$lib/components/ui";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";

  let { description }: { description: string } = $props();
</script>

<EmptyState
  icon="mdi:file-search-outline"
  title="Nothing matches"
  {description}
  action={{ label: "Search everywhere", onclick: () => pageFilterStore.searchEverywhere() }}
/>
