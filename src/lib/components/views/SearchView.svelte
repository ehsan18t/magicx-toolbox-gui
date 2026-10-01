<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { AppCard, TweakCard } from "$lib/components/tweaks";
  import { ActionButton, EmptyState, HighlightedText } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { searchStore, type SearchResult } from "$lib/stores/search.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import {
    applyPendingChanges,
    batchRevertTweaks,
    categoriesStore,
    loadingStateStore,
    loadingStore,
    pendingChangesStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import type { AppView, TweakWithStatus } from "$lib/types";
  import { onDestroy, tick, untrack } from "svelte";

  // Initialize from store to persist across page changes
  let searchInput = $state(searchStore.query);
  let searchInputRef = $state<HTMLInputElement | null>(null);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;
  let scrollRaf: number | null = null;

  // Dialog states
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);

  // Watch for focus signal from navigation store (triggered by Ctrl+K)
  $effect(() => {
    const signal = navigationStore.focusSearchSignal;
    if (signal > 0 && searchInputRef) {
      // Use tick to ensure DOM is ready after navigation
      tick().then(() => searchInputRef?.focus());
    }
  });
  let isBatchProcessing = $state(false);

  // Results are cached per query, so re-run once the app model lands after a search.
  $effect(() => {
    if (appsStore.version > 0) untrack(() => searchStore.isActive && searchStore.search());
  });

  // Search results from store
  const results = $derived(searchStore.results);
  const isSearching = $derived(searchStore.isSearching);
  const error = $derived(searchStore.error);
  const isActive = $derived(searchStore.isActive);

  // Check if tweaks are still loading
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

  /** Mapped search result with tweak or app data and highlight info */
  type MappedResult = { categoryName: string; searchResult: SearchResult } & (
    { kind: "tweak"; tweak: TweakWithStatus } | { kind: "app"; app: AppView }
  );

  const mappedResults = $derived.by((): MappedResult[] => {
    const mapped: MappedResult[] = [];
    for (const result of results) {
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

  const hasResults = $derived(mappedResults.length > 0);
  const resultCount = $derived(mappedResults.length);

  // Apply, discard and restore act on tweak results only.
  const resultTweaks = $derived(mappedResults.flatMap((r) => (r.kind === "tweak" ? [r.tweak] : [])));

  // Tweaks with snapshots (can be restored)
  const tweaksWithSnapshots = $derived(resultTweaks.filter((t) => t.status.has_backup));
  const snapshotCount = $derived(tweaksWithSnapshots.length);

  // Pending changes count for search results
  const searchPendingCount = $derived.by(() => {
    let count = 0;
    const pending = pendingChangesStore.all;
    const tweakIds = new Set(resultTweaks.map((t) => t.definition.id));
    for (const [tweakId] of pending) {
      if (tweakIds.has(tweakId)) {
        count++;
      }
    }
    return count;
  });

  // Loading state
  const isLoading = $derived(resultTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  // Debounced search function
  function handleSearchInput(value: string) {
    searchInput = value;

    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    debounceTimer = setTimeout(() => {
      searchStore.setQuery(value);
    }, 200); // 200ms debounce
  }

  function handleClear() {
    searchInput = "";
    searchStore.clear();
  }

  // Navigate to the item's category and scroll to it
  function navigateToItem({ kind, id, categoryId }: SearchResult) {
    // Set highlight for visual feedback
    searchStore.setHighlight(id);

    // Navigate to the category
    navigationStore.navigateToCategory(categoryId);

    // Scroll to the tweak after a short delay for DOM to update
    if (scrollTimer) {
      clearTimeout(scrollTimer);
      scrollTimer = null;
    }
    if (scrollRaf !== null) {
      cancelAnimationFrame(scrollRaf);
      scrollRaf = null;
    }

    scrollRaf = requestAnimationFrame(() => {
      scrollTimer = setTimeout(() => {
        const element = document.getElementById(`${kind}-${id}`);
        if (element) {
          element.scrollIntoView({ behavior: "smooth", block: "center" });
        }
        scrollTimer = null;
      }, 100);
    });
  }

  async function handleApplyChanges() {
    showApplyAllDialog = false;
    isBatchProcessing = true;
    await applyPendingChanges();
    isBatchProcessing = false;
  }

  async function handleRestoreSnapshots() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    await batchRevertTweaks(tweaksWithSnapshots.map((t) => t.definition.id));

    isBatchProcessing = false;
  }

  function handleRestoreClick() {
    if (snapshotCount === 0) {
      toastStore.info("No snapshots available to restore in search results");
      return;
    }
    showRevertAllDialog = true;
  }

  function handleDiscardChanges() {
    // Clear pending changes for tweaks in search results
    const tweakIds = resultTweaks.map((t) => t.definition.id);
    for (const tweakId of tweakIds) {
      if (pendingChangesStore.has(tweakId)) {
        pendingChangesStore.clear(tweakId);
      }
    }
  }

  // Cleanup debounce timer on component destroy
  onDestroy(() => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }

    if (scrollTimer) {
      clearTimeout(scrollTimer);
      scrollTimer = null;
    }

    if (scrollRaf !== null) {
      cancelAnimationFrame(scrollRaf);
      scrollRaf = null;
    }
  });
</script>

<div class="flex h-full flex-col gap-5 overflow-hidden p-6">
  <!-- Header -->
  <header class="flex flex-wrap items-center justify-between gap-6">
    <div class="flex items-center gap-4">
      <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-accent/15 text-accent">
        <Icon icon="mdi:magnify" width="28" />
      </div>
      <div>
        <h1 class="m-0 text-2xl font-bold tracking-tight text-foreground">Search</h1>
        <p class="mt-1 mb-0 text-sm text-foreground-muted">Find tweaks and apps by name, description, or info</p>
      </div>
    </div>

    {#if hasResults}
      <div class="flex items-center gap-3 rounded-xl border border-border bg-card px-3 py-3">
        <Icon icon="mdi:file-document-multiple" width="36" class="text-accent" />
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-base font-bold text-foreground">{resultCount}</span>
          <span class="text-xs font-semibold text-foreground-muted">Results</span>
        </div>
      </div>
    {/if}
  </header>

  <!-- Search Bar & Toolbar -->
  <div class="flex flex-wrap items-center gap-3">
    <div
      class="flex max-w-full min-w-60 flex-1 items-center gap-2.5 rounded-lg border border-border bg-surface px-4 py-3 transition-all duration-200 focus-within:border-accent focus-within:bg-card"
    >
      {#if isSearching}
        <Icon icon="mdi:loading" width="20" class="animate-spin shrink-0 text-accent" />
      {:else}
        <Icon icon="mdi:magnify" width="20" class="shrink-0 text-foreground-muted" />
      {/if}
      <input
        bind:this={searchInputRef}
        type="text"
        placeholder="Search tweaks and apps... (Ctrl+K)"
        value={searchInput}
        oninput={(e) => handleSearchInput(e.currentTarget.value)}
        class="flex-1 border-0 bg-transparent text-sm text-foreground outline-none placeholder:text-foreground-subtle"
      />
      {#if searchInput}
        <button
          type="button"
          class="hover:bg-muted flex cursor-pointer items-center justify-center rounded border-0 bg-transparent p-1 text-foreground-muted transition-all duration-150 hover:text-foreground"
          onclick={handleClear}
          aria-label="Clear search"
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      {/if}
    </div>

    {#if hasResults}
      <div class="flex gap-2.5">
        <ActionButton
          intent="apply"
          icon="mdi:check-all"
          active={searchPendingCount > 0}
          loading={isBatchProcessing}
          badgeCount={searchPendingCount}
          badgeVariant="warning"
          onclick={() => (showApplyAllDialog = true)}
          disabled={searchPendingCount === 0 || isLoading || isBatchProcessing}
        >
          Apply Changes
        </ActionButton>
        <ActionButton
          intent="discard"
          icon="mdi:close-circle-outline"
          onclick={handleDiscardChanges}
          disabled={searchPendingCount === 0 || isLoading || isBatchProcessing}
          tooltip="Discard all pending changes in search results"
        >
          Discard
        </ActionButton>
        <ActionButton
          intent="restore"
          icon="mdi:restore"
          badgeCount={snapshotCount}
          badgeVariant="error"
          onclick={handleRestoreClick}
          disabled={snapshotCount === 0 || isLoading || isBatchProcessing}
          tooltip={snapshotCount === 0
            ? "No snapshots available"
            : `Restore ${snapshotCount} snapshot${snapshotCount > 1 ? "s" : ""}`}
        >
          Restore Snapshots
        </ActionButton>
      </div>
    {/if}
  </div>

  <!-- Results Area -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if tweaksLoading && !isActive}
      <!-- Loading state -->
      <EmptyState icon="mdi:loading" title="" description="Loading tweaks...">
        <!-- Spinner handled by icon animation -->
      </EmptyState>
    {:else if error}
      <!-- Error state -->
      <EmptyState
        icon="mdi:alert-circle"
        title="Search Error"
        description={error}
        actionText="Retry"
        onaction={() => searchStore.search()}
      />
    {:else if !isActive}
      <!-- Empty state - no search query -->
      <EmptyState
        icon="mdi:text-search"
        title="Start Searching"
        description="Enter a search term to find tweaks and apps across all categories"
      />
    {:else if isSearching}
      <!-- Searching state -->
      <EmptyState icon="mdi:loading" title="" description="Searching..." />
    {:else if !hasResults}
      <!-- No results -->
      <EmptyState
        icon="mdi:file-search-outline"
        title="No results found"
        description={`Nothing matches "${searchStore.query}"`}
        actionText="Clear search"
        onaction={handleClear}
      />
    {:else}
      <!-- Results grid -->
      <div class="flex flex-col gap-3 pb-4 lg:grid lg:grid-cols-2 lg:gap-4">
        {#each mappedResults as result (result.searchResult.id)}
          {@const searchResult = result.searchResult}
          <div class="search-result-card flex flex-col">
            {#if result.kind === "tweak"}
              <TweakCard tweak={result.tweak}>
                {#snippet titleSlot()}
                  {@render highlightedName(result.tweak.definition.name, searchResult)}
                {/snippet}
                {#snippet descriptionSlot()}
                  {@render highlightedDescription(result.tweak.definition.description, searchResult)}
                {/snippet}
              </TweakCard>
            {:else}
              <AppCard app={result.app}>
                {#snippet titleSlot()}
                  {@render highlightedName(result.app.name, searchResult)}
                {/snippet}
                {#snippet descriptionSlot()}
                  {@render highlightedDescription(result.app.description, searchResult)}
                {/snippet}
              </AppCard>
            {/if}
            <!-- Category badge & navigate button at bottom -->
            <div class="mt-auto flex items-center justify-between gap-2 border-t border-border/30 px-4 py-2.5">
              <span
                class="inline-flex items-center gap-1.5 rounded-full bg-accent/10 px-2.5 py-1 text-xs font-medium text-accent"
              >
                <Icon icon="mdi:folder" width="12" />
                {result.categoryName}
              </span>
              <button
                type="button"
                class="flex cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1 text-xs font-medium text-foreground transition-all duration-200 hover:border-accent hover:bg-accent/10 hover:text-accent"
                onclick={() => navigateToItem(searchResult)}
                use:tooltip={"Navigate to location"}
              >
                <Icon icon="mdi:arrow-right-circle" width="14" />
                Go to location
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

{#snippet highlightedName(text: string, searchResult: SearchResult)}
  <HighlightedText
    {text}
    ranges={searchResult.nameRanges}
    highlightClass="bg-accent/25 dark:text-accent-foreground/90 rounded"
  />
{/snippet}

{#snippet highlightedDescription(text: string, searchResult: SearchResult)}
  <HighlightedText
    text={text || ""}
    ranges={searchResult.descriptionRanges}
    highlightClass="bg-accent/25 dark:text-accent-foreground/90 font-semibold rounded"
  />
{/snippet}

<!-- Dialogs -->
<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply Pending Changes"
  message="Apply {searchPendingCount} pending change(s) from search results? Some tweaks may require a system restart."
  confirmText="Apply Changes"
  variant="default"
  onconfirm={handleApplyChanges}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore Snapshots"
  message="Restore {snapshotCount} tweak{snapshotCount > 1 ? 's' : ''} to their original state from saved snapshots?"
  confirmText="Restore Snapshots"
  variant="danger"
  onconfirm={handleRestoreSnapshots}
  oncancel={() => (showRevertAllDialog = false)}
/>

<style lang="postcss">
  @reference "@/app.css";
  .search-result-card {
    @apply overflow-hidden rounded-lg border border-border bg-card transition-all duration-200 hover:border-border-hover hover:shadow-md;
  }
</style>
