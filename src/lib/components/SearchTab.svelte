<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ActionButton, EmptyState, HighlightedText } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { searchStore, type SearchResult } from "$lib/stores/search.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
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
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;
  let scrollRaf: number | null = null;

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

  // Navigate to tweak's category and scroll to it
  function navigateToTweak(tweakId: string, categoryId: string) {
    // Set highlight for visual feedback
    searchStore.setHighlight(tweakId);

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
        const element = document.getElementById(`tweak-${tweakId}`);
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

    let success = 0;
    let failed = 0;

    for (const tweak of tweaksWithSnapshots) {
      // Pass { showToast: false } to suppress individual notifications
      const result = await revertTweak(tweak.definition.id, { showToast: false });
      if (result) {
        success++;
      } else {
        failed++;
      }
    }

    // Show summary toast
    if (failed === 0 && success > 0) {
      toastStore.success(`Restored ${success} snapshot${success > 1 ? "s" : ""} successfully`);
    } else if (failed > 0 && success > 0) {
      toastStore.warning(`Restored ${success}, failed ${failed} snapshot${failed > 1 ? "s" : ""}`);
    } else if (failed > 0) {
      toastStore.error(`Failed to restore ${failed} snapshot${failed > 1 ? "s" : ""}`);
    }

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
        <p class="mt-1 mb-0 text-sm text-foreground-muted">Find tweaks by name, description, or info</p>
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
        description="Enter a search term to find tweaks across all categories"
      />
    {:else if isSearching}
      <!-- Searching state -->
      <EmptyState icon="mdi:loading" title="" description="Searching..." />
    {:else if !hasResults}
      <!-- No results -->
      <EmptyState
        icon="mdi:file-search-outline"
        title="No results found"
        description={`No tweaks match "${searchStore.query}"`}
        actionText="Clear search"
        onaction={handleClear}
      />
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
                use:tooltip={"Navigate to tweak location"}
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
