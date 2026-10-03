<script lang="ts">
  import { LoadError, PendingBar, RebootBanner } from "$lib/components/feedback";
  import { LogsPanel, Sidebar } from "$lib/components/layout";
  import { AppDetailsModal, SummaryPanel, TweakDetailsModal } from "$lib/components/tweaks";
  import {
    CategoryView,
    FavoritesView,
    ManualTestsView,
    OverviewView,
    ProfilesView,
    SearchView,
    SettingsView,
    SnapshotsView,
  } from "$lib/components/views";
  import { bootStore } from "$lib/stores/boot.svelte";
  import { manualTestsStore } from "$lib/stores/manualTests.svelte";
  import { navigationStore, type TabId } from "$lib/stores/navigation.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { errorMessage } from "$lib/utils/error";
  import { logError } from "$lib/utils/logger";
  import { onMount, type Component } from "svelte";

  const SUMMARY_MIN_WIDTH = 1400;

  const PAGE_VIEWS: Partial<Record<TabId, Component>> = {
    overview: OverviewView,
    search: SearchView,
    favorites: FavoritesView,
    snapshots: SnapshotsView,
    profiles: ProfilesView,
    settings: SettingsView,
    "manual-tests": ManualTestsView,
  };

  let loadError = $state<string | null>(null);
  let workspaceWidth = $state(0);

  onMount(async () => {
    void manualTestsStore.load();
    try {
      await bootStore.load();
    } catch (error) {
      loadError = errorMessage(error);
      logError("Failed to initialize", error);
    }
  });

  const activeTab = $derived(navigationStore.activeTab);
  const PageView = $derived(PAGE_VIEWS[activeTab]);
  const categoryTab = $derived(
    navigationStore.isOnCategoryTab ? navigationStore.categoryTabs.find((t) => t.id === activeTab) : undefined,
  );

  const summary = $derived.by(() => {
    if (categoryTab) return { title: categoryTab.name, tweaks: tweaksStore.byCategory[categoryTab.id] ?? [] };
    if (activeTab === "favorites") return { title: "Favorites", tweaks: tweaksStore.favorites };
    if (activeTab === "snapshots") return { title: "Snapshots", tweaks: tweaksStore.withSnapshot };
    return null;
  });
</script>

{#if loadError}
  <div class="flex h-full flex-col">
    <div class="flex min-h-0 flex-1 items-center justify-center p-6">
      <LoadError message={loadError} />
    </div>
    <LogsPanel />
  </div>
{:else}
  <div class="flex h-full min-h-0">
    <Sidebar />
    <main class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-tl-lg border-t border-l border-border bg-surface">
      <RebootBanner />
      <div class="relative flex min-h-0 flex-1" bind:clientWidth={workspaceWidth}>
        <div class="relative flex min-w-0 flex-1 flex-col">
          {#key activeTab}
            <div class="min-h-0 flex-1 animate-rise-in">
              {#if PageView}
                <PageView />
              {:else if categoryTab}
                <CategoryView tab={categoryTab} />
              {/if}
            </div>
          {/key}
          <PendingBar />
        </div>
        {#if summary && workspaceWidth >= SUMMARY_MIN_WIDTH}
          <!-- Keyed: switching pages remounts it rather than animating every group out and in. -->
          {#key summary.title}
            <SummaryPanel label="{summary.title} at a glance" tweaks={summary.tweaks} />
          {/key}
        {/if}
      </div>
      <LogsPanel />
    </main>
  </div>
  <TweakDetailsModal />
  <AppDetailsModal />
{/if}
