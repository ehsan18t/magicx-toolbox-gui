<script lang="ts">
  import { PendingBar, RebootBanner } from "$lib/components/feedback";
  import { Sidebar } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { CategorySummaryPanel, TweakDetailsPanel } from "$lib/components/tweaks";
  import {
    CategoryView,
    FavoritesView,
    ManualTestsView,
    OverviewView,
    ProfileManager,
    SearchView,
    SnapshotsView,
  } from "$lib/components/views";
  import { manualTestsStore } from "$lib/stores/manualTests.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { loadRemainingData } from "$lib/stores/tweaks.svelte";
  import { errorMessage } from "$lib/utils/error";
  import { onMount } from "svelte";

  const PANEL_DOCK_MIN_WIDTH = 1040;
  const SUMMARY_MIN_WIDTH = 1400;

  let error = $state<string | null>(null);
  let workspaceWidth = $state(0);

  onMount(async () => {
    void manualTestsStore.init();
    try {
      await loadRemainingData();
    } catch (e) {
      error = errorMessage(e);
      console.error("Failed to initialize:", e);
    }
  });

  const activeTab = $derived(navigationStore.activeTab);
  const currentCategoryTab = $derived.by(() => {
    if (!navigationStore.isOnCategoryTab) return null;
    return navigationStore.allTabs.find((t: TabDefinition) => t.id === activeTab) ?? null;
  });
</script>

{#if error}
  <div class="flex h-full items-center justify-center p-6">
    <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 text-center">
      <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-error/15 text-error">
        <Icon icon="mdi:alert-circle" width="28" />
      </div>
      <h2 class="mt-4 mb-1 text-base font-semibold">Failed to load</h2>
      <p class="m-0 text-sm wrap-break-word text-foreground-muted">{error}</p>
      <button
        type="button"
        class="mt-5 inline-flex w-full cursor-pointer items-center justify-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-semibold text-accent-foreground hover:bg-accent-hover"
        onclick={() => window.location.reload()}
      >
        <Icon icon="mdi:refresh" width="18" />
        Retry
      </button>
    </div>
  </div>
{:else}
  <div class="flex h-full min-h-0">
    <Sidebar />
    <main class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-tl-lg border-t border-l border-border bg-surface">
      <RebootBanner />
      <div class="relative flex min-h-0 flex-1" bind:clientWidth={workspaceWidth}>
        <div class="relative flex min-w-0 flex-1 flex-col">
          {#key activeTab}
            <div class="min-h-0 flex-1 animate-fade-in">
              {#if activeTab === "overview"}
                <OverviewView />
              {:else if activeTab === "search"}
                <SearchView />
              {:else if activeTab === "favorites"}
                <FavoritesView />
              {:else if activeTab === "snapshots"}
                <SnapshotsView />
              {:else if activeTab === "profiles"}
                <ProfileManager />
              {:else if activeTab === "manual-tests"}
                <ManualTestsView />
              {:else if currentCategoryTab}
                <CategoryView tab={currentCategoryTab} />
              {/if}
            </div>
          {/key}
          <PendingBar />
        </div>
        <TweakDetailsPanel docked={workspaceWidth >= PANEL_DOCK_MIN_WIDTH} />
        {#if currentCategoryTab && workspaceWidth >= SUMMARY_MIN_WIDTH && !tweakDetailsModalStore.isOpen}
          <CategorySummaryPanel tab={currentCategoryTab} />
        {/if}
      </div>
    </main>
  </div>
{/if}
