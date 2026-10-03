<script lang="ts">
  import { PendingBar, RebootBanner } from "$lib/components/feedback";
  import { LogsPanel, Sidebar, SummaryPanel } from "$lib/components/layout";
  import { AppDetailsModal, TweakDetailsModal } from "$lib/components/items";
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
  import { isPageId, navigationStore, type PageId, pageTweaks } from "$lib/stores/navigation.svelte";
  import { remToken } from "$lib/utils/cssToken";
  import type { Component } from "svelte";

  const PAGE_VIEWS: Record<PageId, Component> = {
    overview: OverviewView,
    search: SearchView,
    favorites: FavoritesView,
    snapshots: SnapshotsView,
    profiles: ProfilesView,
    settings: SettingsView,
    "manual-tests": ManualTestsView,
  };

  const activeTab = $derived(navigationStore.activeTab);
  const PageView = $derived(isPageId(activeTab) ? PAGE_VIEWS[activeTab] : undefined);
  const categoryTab = $derived(
    navigationStore.isOnCategoryTab ? navigationStore.categoryTabs.find((t) => t.id === activeTab) : undefined,
  );

  // Mounted only once it fits: hidden by CSS alone, it would still recompute on every status change.
  let workspaceWidth = $state(0);
  const summaryFits = $derived(workspaceWidth >= remToken("--container-summary-panel"));

  const summary = $derived.by(() => {
    const tweaks = pageTweaks(activeTab);
    const title = navigationStore.allTabs.find((t) => t.id === activeTab)?.name;
    return tweaks && title ? { title, tweaks } : null;
  });
</script>

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
      {#if summary && summaryFits}
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
