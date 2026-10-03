<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { SearchInput, ToggleChip } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { searchStore } from "$lib/stores/search.svelte";
  import { tick } from "svelte";
  import { fromAction } from "svelte/attachments";

  let searchEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (navigationStore.focusSearchSignal > 0) tick().then(() => searchEl?.focus());
  });

  const scoped = $derived(pageFilterStore.isScoped);
  const chipTab = $derived(pageFilterStore.chipTab);
  const scopeName = $derived(navigationStore.allTabs.find((t) => t.id === chipTab)?.name ?? "");

  function handleSearch(value: string) {
    if (scoped) pageFilterStore.setQuery(value);
    else if (navigationStore.isScopable) pageFilterStore.searchEverywhere(value);
    else {
      searchStore.setQuery(value);
      if (value && navigationStore.activeTab !== "search") navigationStore.navigateToTab("search");
    }
  }

  function toggleScope() {
    if (scoped) pageFilterStore.searchEverywhere();
    else pageFilterStore.scopeIn();
    searchEl?.focus();
  }
</script>

{#snippet scopeToggle()}
  <ToggleChip
    variant={scoped ? "tint" : "quiet"}
    icon={scoped ? "mdi:filter-variant" : "mdi:filter-variant-remove"}
    class="max-w-9/20 shrink-0 animate-pop-in"
    aria-pressed={scoped}
    aria-label="Search only in {scopeName}"
    {@attach fromAction(tooltip, () =>
      scoped ? `Searching only in ${scopeName}. Select to search everywhere.` : `Search only in ${scopeName}`,
    )}
    onclick={toggleScope}
  >
    <span class="truncate">{scopeName}</span>
  </ToggleChip>
{/snippet}

<SearchInput
  bind:inputRef={searchEl}
  value={scoped ? pageFilterStore.query : searchStore.query}
  placeholder={scoped ? "Filter (Ctrl+K)" : "Search tweaks and apps (Ctrl+K)"}
  label={scoped ? `Filter ${scopeName}` : "Search tweaks and apps"}
  class="w-full max-w-100 drag-disable"
  trailing={chipTab ? scopeToggle : undefined}
  onbackspace={scoped ? toggleScope : undefined}
  onchange={handleSearch}
/>
