<script lang="ts">
  import { HighlightedText } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { searchStore, type SearchResult } from "$lib/stores/search.svelte";
  import {
    applyPendingChanges,
    categoriesStore,
    loadingStateStore,
    loadingStore,
    pendingChangesStore,
    revertTweak,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { onDestroy } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import TweakCard from "./TweakCard.svelte";

  // Initialize from store to persist across page changes
  let searchInput = $state(searchStore.query);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Dialog states
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Search results from store
  const results = $derived(searchStore.results);
  const isSearching = $derived(searchStore.isSearching);
  const hasResults = $derived(searchStore.hasResults);
  const resultCount = $derived(searchStore.resultCount);
  const error = $derived(searchStore.error);
  const isActive = $derived(searchStore.isActive);

  // Check if tweaks are still loading
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

  /** Mapped search result with tweak data and highlight info */
  interface MappedResult {
    tweak: TweakWithStatus;
    categoryName: string;
    searchResult: SearchResult;
  }

  // Map search results to TweakWithStatus for rendering
  const searchResultTweaks = $derived.by((): MappedResult[] => {
    if (!results.length) return [];

    const mappedResults: MappedResult[] = [];

    for (const result of results) {
      const tweak = tweaksStore.getById(result.tweakId);
      if (tweak) {
        const category = categoriesStore.list.find((c) => c.id === result.categoryId);
        mappedResults.push({
          tweak,
          categoryName: category?.name || result.categoryId,
          searchResult: result,
        });
      }
    }

    return mappedResults;
  });

  // Get all tweaks from search results for stats
  const resultTweaks = $derived(searchResultTweaks.map((r) => r.tweak));
  const appliedTweaks = $derived(resultTweaks.filter((t) => t.status.is_applied));

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

  // Navigate to tweak's category and scroll to it
  function navigateToTweak(tweakId: string, categoryId: string) {
    // Set highlight for visual feedback
    searchStore.setHighlight(tweakId);

    // Navigate to the category
    navigationStore.navigateToCategory(categoryId);

    // Scroll to the tweak after a short delay for DOM to update
    requestAnimationFrame(() => {
      setTimeout(() => {
        const element = document.getElementById(`tweak-${tweakId}`);
        if (element) {
          element.scrollIntoView({ behavior: "smooth", block: "center" });
        }
      }, 100);
    });
  }

  async function handleApplyChanges() {
    showApplyAllDialog = false;
    isBatchProcessing = true;
    await applyPendingChanges();
    isBatchProcessing = false;
  }

  async function handleRestoreDefaults() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    for (const tweak of appliedTweaks) {
      await revertTweak(tweak.definition.id);
    }

    isBatchProcessing = false;
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
        <p class="mt-1 mb-0 text-sm text-foreground-muted">Find tweaks by name, description, or info</p>
      </div>
    </div>

    {#if hasResults}
      <div class="flex items-center gap-3 rounded-xl border border-border bg-card px-5 py-3">
        <Icon icon="mdi:file-document-multiple" width="20" class="text-accent" />
        <div class="flex flex-col gap-0.5">
          <span class="text-base font-bold text-foreground">{resultCount}</span>
          <span class="text-xs text-foreground-muted">Results</span>
        </div>
      </div>
    {/if}
  </header>

  <!-- Search Bar & Toolbar -->
  <div class="flex flex-wrap items-center gap-4">
    <div
      class="flex max-w-150 min-w-60 flex-1 items-center gap-2.5 rounded-lg border border-border bg-surface px-4 py-3 transition-all duration-200 focus-within:border-accent focus-within:bg-card"
    >
      {#if isSearching}
        <Icon icon="mdi:loading" width="20" class="animate-spin shrink-0 text-accent" />
      {:else}
        <Icon icon="mdi:magnify" width="20" class="shrink-0 text-foreground-muted" />
      {/if}
      <input
        type="text"
        placeholder="Search tweaks..."
        value={searchInput}
        oninput={(e) => handleSearchInput(e.currentTarget.value)}
        class="flex-1 border-0 bg-transparent text-sm text-foreground outline-none placeholder:text-foreground-subtle"
      />
      {#if searchInput}
        <button
          type="button"
          class="hover:bg-muted flex cursor-pointer items-center justify-center rounded border-0 bg-transparent p-1 text-foreground-muted transition-all duration-150 hover:text-foreground"
          onclick={handleClear}
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      {/if}
    </div>

    {#if hasResults}
      <div class="flex gap-2.5">
        <button
          type="button"
          class="flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-card px-4 py-2.5 text-sm font-medium text-foreground transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-50 {searchPendingCount >
          0
            ? 'border-warning bg-warning/15 text-warning'
            : ''} hover:not-disabled:border-success hover:not-disabled:bg-success/15 hover:not-disabled:text-success"
          onclick={() => (showApplyAllDialog = true)}
          disabled={searchPendingCount === 0 || isLoading || isBatchProcessing}
        >
          {#if isBatchProcessing}
            <Icon icon="mdi:loading" width="18" class="animate-spin" />
          {:else}
            <Icon icon="mdi:check-all" width="18" />
          {/if}
          <span class="hidden sm:inline">Apply Changes</span>
          {#if searchPendingCount > 0}
            <span
              class="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-warning px-1.5 text-xs font-bold text-white"
              >{searchPendingCount}</span
            >
          {/if}
        </button>
        <button
          type="button"
          class="flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-card px-4 py-2.5 text-sm font-medium text-foreground transition-all duration-200 hover:not-disabled:border-foreground-muted hover:not-disabled:bg-foreground/5 disabled:cursor-not-allowed disabled:opacity-50"
          onclick={handleDiscardChanges}
          disabled={searchPendingCount === 0 || isLoading || isBatchProcessing}
          title="Discard all pending changes in search results"
        >
          <Icon icon="mdi:close-circle-outline" width="18" />
          <span class="hidden sm:inline">Discard</span>
        </button>
        <button
          type="button"
          class="flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-card px-4 py-2.5 text-sm font-medium text-foreground transition-all duration-200 hover:not-disabled:border-error hover:not-disabled:bg-error/15 hover:not-disabled:text-error disabled:cursor-not-allowed disabled:opacity-50"
          onclick={() => (showRevertAllDialog = true)}
          disabled={appliedTweaks.length === 0 || isLoading || isBatchProcessing}
        >
          <Icon icon="mdi:restore" width="18" />
          <span class="hidden sm:inline">Restore Defaults</span>
        </button>
      </div>
    {/if}
  </div>

  <!-- Results Area -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if tweaksLoading && !isActive}
      <!-- Loading state -->
      <div class="flex flex-col items-center justify-center gap-3 px-6 py-15 text-center text-foreground-muted">
        <Icon icon="mdi:loading" width="48" class="animate-spin text-accent" />
        <p class="m-0 text-sm">Loading tweaks...</p>
      </div>
    {:else if error}
      <!-- Error state -->
      <div class="flex flex-col items-center justify-center gap-3 px-6 py-15 text-center text-foreground-muted">
        <Icon icon="mdi:alert-circle" width="56" class="text-error" />
        <h3 class="m-0 text-lg font-semibold text-foreground">Search Error</h3>
        <p class="m-0 text-sm">{error}</p>
        <button
          type="button"
          class="mt-2 cursor-pointer rounded-lg border-0 bg-accent px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:brightness-110"
          onclick={() => searchStore.search()}
        >
          Retry
        </button>
      </div>
    {:else if !isActive}
      <!-- Empty state - no search query -->
      <div class="flex flex-col items-center justify-center gap-3 px-6 py-15 text-center text-foreground-muted">
        <Icon icon="mdi:text-search" width="56" />
        <h3 class="m-0 text-lg font-semibold text-foreground">Start Searching</h3>
        <p class="m-0 text-sm">Enter a search term to find tweaks across all categories</p>
      </div>
    {:else if isSearching}
      <!-- Searching state -->
      <div class="flex flex-col items-center justify-center gap-3 px-6 py-15 text-center text-foreground-muted">
        <Icon icon="mdi:loading" width="48" class="animate-spin text-accent" />
        <p class="m-0 text-sm">Searching...</p>
      </div>
    {:else if !hasResults}
      <!-- No results -->
      <div class="flex flex-col items-center justify-center gap-3 px-6 py-15 text-center text-foreground-muted">
        <Icon icon="mdi:file-search-outline" width="56" />
        <h3 class="m-0 text-lg font-semibold text-foreground">No results found</h3>
        <p class="m-0 text-sm">No tweaks match "{searchStore.query}"</p>
        <button
          type="button"
          class="mt-2 cursor-pointer rounded-lg border-0 bg-accent px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:brightness-110"
          onclick={handleClear}
        >
          Clear search
        </button>
      </div>
    {:else}
      <!-- Results grid -->
      <div class="flex flex-col gap-3 pb-4 lg:grid lg:grid-cols-2 lg:gap-4">
        {#each searchResultTweaks as { tweak, categoryName, searchResult } (tweak.definition.id)}
          <div class="search-result-card flex flex-col">
            <TweakCard {tweak}>
              {#snippet titleSlot()}
                <HighlightedText
                  text={tweak.definition.name}
                  ranges={searchResult.nameRanges}
                  highlightClass="bg-accent/25 dark:text-accent-foreground/90 rounded"
                />
              {/snippet}
              {#snippet descriptionSlot()}
                <HighlightedText
                  text={tweak.definition.description || ""}
                  ranges={searchResult.descriptionRanges}
                  highlightClass="bg-accent/25 dark:text-accent-foreground/90 font-semibold rounded"
                />
              {/snippet}
            </TweakCard>
            <!-- Category badge & navigate button at bottom -->
            <div class="mt-auto flex items-center justify-between gap-2 border-t border-border/30 px-4 py-2.5">
              <span
                class="inline-flex items-center gap-1.5 rounded-full bg-accent/10 px-2.5 py-1 text-xs font-medium text-accent"
              >
                <Icon icon="mdi:folder" width="12" />
                {categoryName}
              </span>
              <button
                type="button"
                class="flex cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1 text-xs font-medium text-foreground transition-all duration-200 hover:border-accent hover:bg-accent/10 hover:text-accent"
                onclick={() => navigateToTweak(tweak.definition.id, tweak.definition.category_id)}
                title="Navigate to tweak location"
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
  title="Restore System Defaults"
  message="Restore {appliedTweaks.length} applied tweak(s) from search results to their original system values?"
  confirmText="Restore Defaults"
  variant="danger"
  onconfirm={handleRestoreDefaults}
  oncancel={() => (showRevertAllDialog = false)}
/>

<style>
  @reference "@/app.css";
  .search-result-card {
    @apply overflow-hidden rounded-lg border border-border bg-card transition-all duration-200 hover:border-border-hover hover:shadow-md;
  }
</style>
