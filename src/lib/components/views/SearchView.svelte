<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { AppRow, TweakRow } from "$lib/components/tweaks";
  import { EmptyState, HighlightedText } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { searchStore, type SearchResult } from "$lib/stores/search.svelte";
  import {
    batchRevertTweaks,
    categoriesStore,
    loadingStateStore,
    loadingStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import type { AppView, TweakWithStatus } from "$lib/types";
  import { untrack } from "svelte";

  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Results are cached per query, so re-run once the app model lands after a search.
  $effect(() => {
    if (appsStore.version > 0) untrack(() => searchStore.isActive && searchStore.search());
  });

  type MappedResult = { categoryName: string; searchResult: SearchResult } & (
    { kind: "tweak"; tweak: TweakWithStatus } | { kind: "app"; app: AppView }
  );

  const mappedResults = $derived.by((): MappedResult[] => {
    const mapped: MappedResult[] = [];
    for (const result of searchStore.results) {
      const categoryName = categoriesStore.getName(result.categoryId);
      if (result.kind === "tweak") {
        const tweak = tweaksStore.getById(result.id);
        if (tweak) mapped.push({ kind: "tweak", tweak, categoryName, searchResult: result });
      } else if (appsStore.isVisible(result.id)) {
        const app = appsStore.list.find((a) => a.id === result.id);
        if (app) mapped.push({ kind: "app", app, categoryName, searchResult: result });
      }
    }
    return mapped;
  });

  const resultTweaks = $derived(mappedResults.flatMap((r) => (r.kind === "tweak" ? [r.tweak] : [])));
  const tweaksWithSnapshots = $derived(resultTweaks.filter((t) => t.status.has_backup));
  const isLoading = $derived(resultTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  const description = $derived(
    searchStore.isActive && !searchStore.isSearching
      ? `${mappedResults.length} result${mappedResults.length === 1 ? "" : "s"} for "${searchStore.query.trim()}"`
      : "Find tweaks and apps by name, description or details.",
  );

  function goToItem({ id, categoryId }: SearchResult) {
    searchStore.setHighlight(id);
    navigationStore.navigateToCategory(categoryId);
  }

  async function handleRestoreSnapshots() {
    showRevertAllDialog = false;
    isBatchProcessing = true;
    await batchRevertTweaks(tweaksWithSnapshots.map((t) => t.definition.id));
    isBatchProcessing = false;
  }
</script>

{#snippet highlighted(text: string, ranges: number[])}
  <HighlightedText text={text || ""} {ranges} highlightClass="rounded-sm bg-accent/25 text-foreground" />
{/snippet}

{#snippet location(result: MappedResult)}
  <span class="inline-flex items-center gap-1 text-foreground-muted">
    <Icon icon={categoriesStore.getIcon(result.searchResult.categoryId)} width="13" class="shrink-0" />
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
  {#snippet toolbar()}
    {#if tweaksWithSnapshots.length > 0}
      <button
        type="button"
        class="ml-auto inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
        disabled={isLoading || isBatchProcessing}
        onclick={() => (showRevertAllDialog = true)}
      >
        <Icon icon="mdi:history" width="16" />
        Restore results
        <span class="text-xs text-foreground-subtle tabular-nums">{tweaksWithSnapshots.length}</span>
      </button>
    {/if}
  {/snippet}

  {#if loadingStateStore.tweaksLoading && !searchStore.isActive}
    <EmptyState icon="mdi:loading" title="" description="Loading tweaks…" />
  {:else if searchStore.error}
    <EmptyState
      icon="mdi:alert-circle"
      title="Search failed"
      description={searchStore.error}
      actionText="Retry"
      onaction={() => searchStore.search()}
    />
  {:else if !searchStore.isActive}
    <EmptyState
      icon="mdi:text-search"
      title="Start typing to search"
      description="Use the search box in the title bar, or press Ctrl+K from anywhere."
      actionText="Focus search"
      onaction={() => navigationStore.focusSearch()}
    />
  {:else if searchStore.isSearching}
    <EmptyState icon="mdi:loading" title="" description="Searching…" />
  {:else if mappedResults.length === 0}
    <EmptyState
      icon="mdi:file-search-outline"
      title="No results"
      description={`Nothing matches "${searchStore.query.trim()}"`}
      actionText="Clear search"
      onaction={() => searchStore.setQuery("")}
    />
  {:else}
    <div class="flex flex-col gap-2">
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

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore Snapshots"
  message="Restore {tweaksWithSnapshots.length} tweak{tweaksWithSnapshots.length === 1
    ? ''
    : 's'} from these results to their original state from saved snapshots?"
  confirmText="Restore Snapshots"
  variant="danger"
  onconfirm={handleRestoreSnapshots}
  oncancel={() => (showRevertAllDialog = false)}
/>
