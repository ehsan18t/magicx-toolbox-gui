<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { ICON_SIZE, SearchInput } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { searchStore } from "$lib/stores/search.svelte";
  import { tick } from "svelte";

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
  <button
    type="button"
    class="inline-flex max-w-9/20 shrink-0 animate-pop-in cursor-pointer items-center gap-1 rounded border px-1.5 py-0.5 text-xs font-medium {scoped
      ? 'border-accent/40 bg-accent/15 text-accent hover:bg-accent/20'
      : 'border-border-hover bg-muted text-foreground-muted hover:border-accent/50 hover:text-foreground'}"
    aria-pressed={scoped}
    aria-label="Search only in {scopeName}"
    use:tooltip={scoped
      ? `Searching only in ${scopeName}. Select to search everywhere.`
      : `Search only in ${scopeName}`}
    onclick={toggleScope}
  >
    <Icon icon={scoped ? "mdi:filter-variant" : "mdi:filter-variant-remove"} width={ICON_SIZE.xs} class="shrink-0" />
    <span class="truncate">{scopeName}</span>
  </button>
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
