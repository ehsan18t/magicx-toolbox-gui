<script lang="ts">
  import { CategorySection, FilterBar, StatsCard, SystemInfoCard, TweakCard } from "$lib";
  import {
    filteredTweaks,
    initializeStores,
    selectedCategory,
    systemStore,
    tweaksByCategory,
    tweakStats,
  } from "$lib/stores/tweaks";
  import { type TweakCategory } from "$lib/types";
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";

  let loading = $state(true);
  let error = $state<string | null>(null);

  const categories: TweakCategory[] = [
    "privacy",
    "performance",
    "ui",
    "security",
    "services",
    "gaming",
  ];

  onMount(async () => {
    try {
      await initializeStores();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load data";
      console.error("Failed to initialize:", e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="page-container">
  {#if loading}
    <div class="loading-screen">
      <Icon icon="mdi:loading" width="48" class="spin" />
      <p>Loading tweaks...</p>
    </div>
  {:else if error}
    <div class="error-screen">
      <Icon icon="mdi:alert-circle" width="48" class="error-icon" />
      <h2>Failed to Load</h2>
      <p>{error}</p>
      <button onclick={() => window.location.reload()}>
        <Icon icon="mdi:refresh" width="18" />
        Retry
      </button>
    </div>
  {:else}
    <header class="page-header">
      <div class="header-content">
        <h1>
          <Icon icon="mdi:tune-vertical" width="28" />
          Windows Tweaks
        </h1>
        <p class="subtitle">
          Optimize your Windows experience with {$tweakStats.total} available tweaks
        </p>
      </div>
    </header>

    <section class="info-section">
      <SystemInfoCard systemInfo={$systemStore} />
      <StatsCard
        total={$tweakStats.total}
        applied={$tweakStats.applied}
        pending={$tweakStats.pending}
      />
    </section>

    <section class="tweaks-section">
      <FilterBar />

      {#if $selectedCategory === "all"}
        <!-- Show all categories -->
        {#each categories as category}
          {@const categoryTweaks = $tweaksByCategory[category]}
          {#if categoryTweaks.length > 0}
            <CategorySection {category} tweaks={categoryTweaks} />
          {/if}
        {/each}
      {:else}
        <!-- Show filtered results -->
        {#if $filteredTweaks.length > 0}
          <div class="filtered-tweaks">
            {#each $filteredTweaks as tweak (tweak.definition.id)}
              <TweakCard {tweak} />
            {/each}
          </div>
        {:else}
          <div class="no-results">
            <Icon icon="mdi:magnify-close" width="48" class="no-results-icon" />
            <p>No tweaks found matching your criteria</p>
          </div>
        {/if}
      {/if}
    </section>

    <footer class="page-footer">
      <p>
        <Icon icon="mdi:information" width="14" />
        Changes are backed up automatically. Reboot may be required for some tweaks.
      </p>
    </footer>
  {/if}
</div>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    padding: 16px;
    background: hsl(var(--background));
  }

  .loading-screen,
  .error-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 16px;
    color: hsl(var(--muted-foreground));
  }

  .error-screen h2 {
    color: hsl(var(--foreground));
    margin: 0;
  }

  :global(.error-icon) {
    color: hsl(0 84% 60%);
  }

  .error-screen button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    border: none;
    border-radius: 8px;
    background: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .error-screen button:hover {
    opacity: 0.9;
  }

  .page-header {
    margin-bottom: 16px;
  }

  .header-content h1 {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 24px;
    font-weight: 700;
    color: hsl(var(--foreground));
    margin: 0;
  }

  .subtitle {
    font-size: 14px;
    color: hsl(var(--muted-foreground));
    margin: 4px 0 0 0;
  }

  .info-section {
    display: grid;
    gap: 16px;
    margin-bottom: 16px;
  }

  @media (min-width: 900px) {
    .info-section {
      grid-template-columns: 1fr auto;
    }
  }

  .tweaks-section {
    flex: 1;
  }

  .filtered-tweaks {
    display: grid;
    gap: 8px;
  }

  @media (min-width: 768px) {
    .filtered-tweaks {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (min-width: 1200px) {
    .filtered-tweaks {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .no-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 16px;
    color: hsl(var(--muted-foreground));
    text-align: center;
  }

  :global(.no-results-icon) {
    opacity: 0.5;
    margin-bottom: 8px;
  }

  .page-footer {
    margin-top: 24px;
    padding-top: 16px;
    border-top: 1px solid hsl(var(--border));
  }

  .page-footer p {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-size: 12px;
    color: hsl(var(--muted-foreground));
    margin: 0;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
