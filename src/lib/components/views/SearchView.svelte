<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { AppRow, MetaItem, TweakRow } from "$lib/components/items";
  import { EmptyState, LinkButton } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { type SearchResult, searchStore } from "$lib/stores/search.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import type { AppView, TweakWithStatus } from "$lib/types";
  import { plural } from "$lib/utils/format";

  type MappedResult = { name: string; categoryName: string; searchResult: SearchResult } & (
    { kind: "tweak"; tweak: TweakWithStatus } | { kind: "app"; app: AppView }
  );

  const mappedResults = $derived.by((): MappedResult[] => {
    const mapped: MappedResult[] = [];
    for (const searchResult of searchStore.results) {
      const categoryName = categoriesStore.name(searchResult.categoryId);
      if (searchResult.kind === "tweak") {
        const tweak = tweaksStore.tweak(searchResult.id);
        if (tweak) mapped.push({ kind: "tweak", tweak, name: tweak.definition.name, categoryName, searchResult });
      } else if (appsStore.isVisible(searchResult.id)) {
        const app = appsStore.app(searchResult.id);
        if (app) mapped.push({ kind: "app", app, name: app.name, categoryName, searchResult });
      }
    }
    return mapped;
  });

  const description = $derived(
    searchStore.isActive && searchStore.searchedQuery
      ? `${plural(mappedResults.length, "result")} for "${searchStore.searchedQuery}"`
      : "Find tweaks and apps by name, description or details.",
  );

  function goToItem({ id, categoryId }: SearchResult) {
    searchStore.setHighlight(id);
    navigationStore.navigateToTab(categoryId);
  }
</script>

{#snippet location(result: MappedResult)}
  <MetaItem icon={categoriesStore.icon(result.searchResult.categoryId)} label={result.categoryName} tone="neutral" />
  <LinkButton
    variant="hover"
    tone="accent"
    class="inline-flex items-center gap-1"
    aria-label="Go to {result.name} in {result.categoryName}"
    onclick={() => goToItem(result.searchResult)}
  >
    Go to
    <Icon icon="mdi:arrow-right" size="2xs" />
  </LinkButton>
{/snippet}

<PageLayout title="Search" {description} announce>
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
    <h2 class="sr-only">Results</h2>
    <div class="flex animate-fade-in flex-col gap-2">
      {#each mappedResults as result (result.searchResult.id)}
        {#if result.kind === "tweak"}
          <TweakRow tweak={result.tweak} match={result.searchResult}>
            {#snippet context()}{@render location(result)}{/snippet}
          </TweakRow>
        {:else}
          <AppRow app={result.app} match={result.searchResult}>
            {#snippet context()}{@render location(result)}{/snippet}
          </AppRow>
        {/if}
      {/each}
    </div>
  {/if}
</PageLayout>
