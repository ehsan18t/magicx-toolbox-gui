<script lang="ts">
  import CategoryTab from "$lib/components/CategoryTab.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import OverviewTab from "$lib/components/OverviewTab.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { categoriesStore, loadRemainingData } from "$lib/stores/tweaks.svelte";
  import { onMount } from "svelte";

  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      // Categories may already be loaded by layout - only load remaining data
      // This avoids duplicate network requests
      if (categoriesStore.list.length > 0) {
        // Layout already loaded categories, just load remaining data
        await loadRemainingData();
      } else {
        // Fallback: load everything if layout hasn't started yet
        const { initializeData } = await import("$lib/stores/tweaksData.svelte");
        await initializeData();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load data";
      console.error("Failed to initialize:", e);
    }
  });

  // Derived values from navigation store
  const activeTab = $derived(navigationStore.activeTab);
  const allTabs = $derived(navigationStore.allTabs);

  // Get the current tab definition for CategoryTab
  const currentCategoryTab = $derived.by(() => {
    if (activeTab === "overview") return null;
    return allTabs.find((t: TabDefinition) => t.id === activeTab) ?? null;
  });
</script>

<div class="page-container">
  {#if error}
    <div class="error-screen">
      <div class="error-content">
        <div class="error-icon-wrapper">
          <Icon icon="mdi:alert-circle" width="48" />
        </div>
        <h2>Failed to Load</h2>
        <p class="error-message">{error}</p>
        <button class="retry-button" onclick={() => window.location.reload()}>
          <Icon icon="mdi:refresh" width="18" />
          Retry
        </button>
      </div>
    </div>
  {:else}
    <!-- Always show app shell - components handle their own loading states -->
    <div class="app-layout">
      <Sidebar />
      <main class="main-content">
        <div class="content-area">
          {#if activeTab === "overview"}
            <OverviewTab />
          {:else if currentCategoryTab}
            <CategoryTab tab={currentCategoryTab} />
          {/if}
        </div>
      </main>
    </div>
  {/if}
</div>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: hsl(var(--background));
  }

  /* Error Screen */
  .error-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
  }

  .error-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
    max-width: 360px;
    padding: 32px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 16px;
  }

  .error-icon-wrapper {
    width: 72px;
    height: 72px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: hsl(0 84% 60% / 0.1);
    border-radius: 50%;
    color: hsl(0 84% 60%);
  }

  .error-content h2 {
    margin: 8px 0 0;
    font-size: 18px;
    font-weight: 600;
    color: hsl(var(--foreground));
  }

  .error-message {
    margin: 0;
    font-size: 14px;
    line-height: 1.5;
    color: hsl(var(--muted-foreground));
  }

  .retry-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    padding: 10px 20px;
    border: none;
    border-radius: 10px;
    background: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .retry-button:hover {
    background: hsl(var(--primary) / 0.9);
    transform: translateY(-1px);
  }

  /* Main App Layout */
  .app-layout {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .content-area {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
