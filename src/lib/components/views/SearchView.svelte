<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { AppRow, TweakRow } from "$lib/components/tweaks";
  import { EmptyState, HighlightedText } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { type SearchResult, searchStore } from "$lib/stores/search.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import type { AppView, TweakWithStatus } from "$lib/types";

  type MappedResult = { categoryName: string; searchResult: SearchResult } & (
    { kind: "tweak"; tweak: TweakWithStatus } | { kind: "app"; app: AppView }
  );

  const mappedResults = $derived.by((): MappedResult[] => {
    const mapped: MappedResult[] = [];
    const byId = new Map(tweaksStore.list.map((t) => [t.definition.id, t]));
    for (const result of searchStore.results) {
      const categoryName = categoriesStore.name(result.categoryId);
      if (result.kind === "tweak") {
        const tweak = byId.get(result.id);
        if (tweak) mapped.push({ kind: "tweak", tweak, categoryName, searchResult: result });
      } else if (appsStore.isVisible(result.id)) {
        const app = appsStore.list.find((a) => a.id === result.id);
        if (app) mapped.push({ kind: "app", app, categoryName, searchResult: result });
      }
    }
    return mapped;
  });

  const description = $derived(
    searchStore.isActive && searchStore.searchedQuery
      ? `${mappedResults.length} result${mappedResults.length === 1 ? "" : "s"} for "${searchStore.searchedQuery}"`
      : "Find tweaks and apps by name, description or details.",
  );

  function goToItem({ id, categoryId }: SearchResult) {
    searchStore.setHighlight(id);
    navigationStore.navigateToTab(categoryId);
  }
</script>

{#snippet highlighted(text: string, ranges: number[])}
  <HighlightedText text={text || ""} {ranges} />
{/snippet}

{#snippet location(result: MappedResult)}
  <span class="inline-flex items-center gap-1 text-foreground-muted">
    <Icon icon={categoriesStore.icon(result.searchResult.categoryId)} width="13" class="shrink-0" />
    {result.categoryName}
  </span>
  <button
    type="button"
    class="inline-flex cursor-pointer items-center gap-1 rounded px-1 text-accent hover:underline"
    aria-label="Go to {result.kind === 'tweak'
      ? result.tweak.definition.name
      : result.app.name} in {result.categoryName}"
    onclick={() => goToItem(result.searchResult)}
  >
    Go to
    <Icon icon="mdi:arrow-right" width="13" />
  </button>
{/snippet}

<PageLayout title="Search" {description}>
  {#if tweaksStore.isLoading && !searchStore.isActive}
    <EmptyState loading description="Loading tweaks…" />
  {:else if searchStore.error}
    <EmptyState
      icon="mdi:alert-circle"
      title="Search failed"
      description={searchStore.error}
      action={{ label: "Retry", onclick: () => searchStore.search() }}
    />
  {:else if !searchStore.isActive}
    <EmptyState
      icon="mdi:text-search"
      title="Start typing to search"
      description="Use the search box in the title bar, or press Ctrl+K from anywhere."
      action={{ label: "Focus search", onclick: () => navigationStore.focusSearch() }}
    />
  {:else if !searchStore.searchedQuery}
    <!-- The first search is still debouncing: show nothing rather than a false "No results". -->
  {:else if mappedResults.length === 0}
    <EmptyState
      icon="mdi:file-search-outline"
      title="No results"
      description={`Nothing matches "${searchStore.searchedQuery}"`}
      action={{ label: "Clear search", onclick: () => searchStore.setQuery("") }}
    />
  {:else}
    <div class="flex animate-fade-in flex-col gap-2">
      {#each mappedResults as result (result.searchResult.id)}
        {@const ranges = result.searchResult}
        {#if result.kind === "tweak"}
          <TweakRow tweak={result.tweak}>
            {#snippet titleSlot()}{@render highlighted(result.tweak.definition.name, ranges.nameRanges)}{/snippet}
            {#snippet descriptionSlot()}
              {@render highlighted(result.tweak.definition.description, ranges.descriptionRanges)}
            {/snippet}
            {#snippet context()}{@render location(result)}{/snippet}
          </TweakRow>
        {:else}
          <AppRow app={result.app}>
            {#snippet titleSlot()}{@render highlighted(result.app.name, ranges.nameRanges)}{/snippet}
            {#snippet descriptionSlot()}{@render highlighted(
                result.app.description,
                ranges.descriptionRanges,
              )}{/snippet}
            {#snippet context()}{@render location(result)}{/snippet}
          </AppRow>
        {/if}
      {/each}
    </div>
  {/if}
</PageLayout>
