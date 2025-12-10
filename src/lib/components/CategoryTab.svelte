<script lang="ts">
  import type { TabDefinition } from "$lib/stores/navigation";
  import { applyTweak, loadingStore, revertTweak, tweaksStore } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";
  import { derived } from "svelte/store";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import TweakCard from "./TweakCard.svelte";

  interface Props {
    tab: TabDefinition;
  }

  const { tab }: Props = $props();

  let searchQuery = $state("");
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Get tweaks for this category
  const categoryTweaks = $derived($tweaksStore.filter((t) => t.definition.category === tab.id));

  // Filter tweaks by search - use $derived.by for computed with logic
  const filteredTweaks = $derived.by(() => {
    if (!searchQuery.trim()) return categoryTweaks;
    const query = searchQuery.toLowerCase();
    return categoryTweaks.filter(
      (t) =>
        t.definition.name.toLowerCase().includes(query) ||
        t.definition.description.toLowerCase().includes(query),
    );
  });

  // Stats
  const appliedCount = $derived(categoryTweaks.filter((t) => t.status.is_applied).length);
  const unappliedTweaks = $derived(categoryTweaks.filter((t) => !t.status.is_applied));
  const appliedTweaks = $derived(categoryTweaks.filter((t) => t.status.is_applied));
  const totalCount = $derived(categoryTweaks.length);
  const progressPercent = $derived(
    totalCount > 0 ? Math.round((appliedCount / totalCount) * 100) : 0,
  );

  // Loading state
  const isLoading = derived(loadingStore, ($loading) =>
    categoryTweaks.some((t) => $loading.has(t.definition.id)),
  );

  async function handleApplyAll() {
    showApplyAllDialog = false;
    isBatchProcessing = true;

    for (const tweak of unappliedTweaks) {
      await applyTweak(tweak.definition.id);
    }

    isBatchProcessing = false;
  }

  async function handleRevertAll() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    for (const tweak of appliedTweaks) {
      await revertTweak(tweak.definition.id);
    }

    isBatchProcessing = false;
  }
</script>

<div class="category-page">
  <!-- Header -->
  <header class="page-header">
    <div class="header-left">
      <div class="header-icon">
        <Icon icon={tab.icon || "mdi:folder"} width="28" />
      </div>
      <div class="header-info">
        <h1>{tab.name}</h1>
        <p>{tab.description}</p>
      </div>
    </div>

    <div class="header-stats">
      <div class="stat-ring" style="--progress: {progressPercent}">
        <svg viewBox="0 0 36 36">
          <circle class="bg" cx="18" cy="18" r="14" />
          <circle class="progress" cx="18" cy="18" r="14" />
        </svg>
        <span>{progressPercent}%</span>
      </div>
      <div class="stat-text">
        <span class="stat-main">{appliedCount} / {totalCount}</span>
        <span class="stat-label">Applied</span>
      </div>
    </div>
  </header>

  <!-- Toolbar -->
  <div class="toolbar">
    <div class="search-box">
      <Icon icon="mdi:magnify" width="20" />
      <input type="text" placeholder="Search tweaks..." bind:value={searchQuery} />
      {#if searchQuery}
        <button class="clear-btn" onclick={() => (searchQuery = "")}>
          <Icon icon="mdi:close" width="16" />
        </button>
      {/if}
    </div>

    <div class="toolbar-actions">
      <button
        class="action-btn apply"
        onclick={() => (showApplyAllDialog = true)}
        disabled={unappliedTweaks.length === 0 || $isLoading || isBatchProcessing}
      >
        {#if isBatchProcessing}
          <Icon icon="mdi:loading" width="18" class="spin" />
        {:else}
          <Icon icon="mdi:check-all" width="18" />
        {/if}
        <span>Apply All</span>
      </button>
      <button
        class="action-btn revert"
        onclick={() => (showRevertAllDialog = true)}
        disabled={appliedTweaks.length === 0 || $isLoading || isBatchProcessing}
      >
        <Icon icon="mdi:undo-variant" width="18" />
        <span>Revert All</span>
      </button>
    </div>
  </div>

  <!-- Tweaks Grid -->
  <div class="tweaks-container">
    {#if filteredTweaks.length === 0}
      <div class="empty-state">
        {#if searchQuery}
          <Icon icon="mdi:file-search-outline" width="56" />
          <h3>No results found</h3>
          <p>No tweaks match "{searchQuery}"</p>
          <button class="clear-search-btn" onclick={() => (searchQuery = "")}>
            Clear search
          </button>
        {:else}
          <Icon icon="mdi:package-variant" width="56" />
          <h3>No tweaks available</h3>
          <p>This category has no tweaks for your system</p>
        {/if}
      </div>
    {:else}
      <div class="tweaks-grid">
        {#each filteredTweaks as tweak (tweak.definition.id)}
          <TweakCard {tweak} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Dialogs -->
<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply All Tweaks"
  message="Are you sure you want to apply all {unappliedTweaks.length} unapplied tweaks? Some may require a system restart."
  confirmText="Apply All"
  variant="default"
  onconfirm={handleApplyAll}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Revert All Tweaks"
  message="Are you sure you want to revert all {appliedTweaks.length} applied tweaks to default values?"
  confirmText="Revert All"
  variant="danger"
  onconfirm={handleRevertAll}
  oncancel={() => (showRevertAllDialog = false)}
/>

<style>
  .category-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 24px;
    gap: 20px;
    overflow: hidden;
  }

  /* Header */
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    flex-wrap: wrap;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .header-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    background: hsl(var(--accent) / 0.15);
    color: hsl(var(--accent));
    border-radius: 16px;
    flex-shrink: 0;
  }

  .header-info h1 {
    font-size: 24px;
    font-weight: 700;
    color: hsl(var(--foreground));
    margin: 0;
    letter-spacing: -0.3px;
  }

  .header-info p {
    font-size: 14px;
    color: hsl(var(--foreground-muted));
    margin: 4px 0 0 0;
  }

  .header-stats {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 12px;
  }

  .stat-ring {
    position: relative;
    width: 40px;
    height: 40px;
  }

  .stat-ring svg {
    transform: rotate(-90deg);
    width: 100%;
    height: 100%;
  }

  .stat-ring circle {
    fill: none;
    stroke-width: 3;
    stroke-linecap: round;
  }

  .stat-ring .bg {
    stroke: hsl(var(--muted));
  }

  .stat-ring .progress {
    stroke: hsl(var(--accent));
    stroke-dasharray: 88;
    stroke-dashoffset: calc(88 * (1 - var(--progress) / 100));
    transition: stroke-dashoffset 0.4s ease;
  }

  .stat-ring span {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 10px;
    font-weight: 700;
    color: hsl(var(--foreground));
  }

  .stat-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-main {
    font-size: 16px;
    font-weight: 700;
    color: hsl(var(--foreground));
  }

  .stat-label {
    font-size: 11px;
    color: hsl(var(--foreground-muted));
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 240px;
    max-width: 400px;
    padding: 10px 16px;
    background: hsl(var(--surface));
    border: 1px solid hsl(var(--border));
    border-radius: 10px;
    transition: all 0.2s ease;
  }

  .search-box:focus-within {
    border-color: hsl(var(--accent));
    background: hsl(var(--card));
  }

  .search-box > :global(svg) {
    color: hsl(var(--foreground-muted));
    flex-shrink: 0;
  }

  .search-box input {
    flex: 1;
    background: transparent;
    border: none;
    color: hsl(var(--foreground));
    font-size: 14px;
    outline: none;
  }

  .search-box input::placeholder {
    color: hsl(var(--foreground-subtle));
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    background: transparent;
    border: none;
    color: hsl(var(--foreground-muted));
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .clear-btn:hover {
    color: hsl(var(--foreground));
    background: hsl(var(--muted));
  }

  .toolbar-actions {
    display: flex;
    gap: 10px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    font-size: 13px;
    font-weight: 500;
    border: 1px solid hsl(var(--border));
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.2s ease;
    background: hsl(var(--card));
    color: hsl(var(--foreground));
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.apply:not(:disabled):hover {
    background: hsl(var(--success) / 0.15);
    border-color: hsl(var(--success));
    color: hsl(var(--success));
  }

  .action-btn.revert:not(:disabled):hover {
    background: hsl(var(--error) / 0.15);
    border-color: hsl(var(--error));
    color: hsl(var(--error));
  }

  .action-btn span {
    display: none;
  }

  @media (min-width: 640px) {
    .action-btn span {
      display: inline;
    }
  }

  /* Tweaks Container */
  .tweaks-container {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    padding-right: 8px;
    margin-right: -8px;
  }

  .tweaks-grid {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 16px;
  }

  @media (min-width: 1024px) {
    .tweaks-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 16px;
    }
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px 24px;
    text-align: center;
    color: hsl(var(--foreground-muted));
  }

  .empty-state h3 {
    font-size: 18px;
    font-weight: 600;
    color: hsl(var(--foreground));
    margin: 0;
  }

  .empty-state p {
    font-size: 14px;
    margin: 0;
  }

  .clear-search-btn {
    margin-top: 8px;
    padding: 10px 20px;
    background: hsl(var(--accent));
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .clear-search-btn:hover {
    filter: brightness(1.1);
  }

  /* Animations */
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

  /* Scrollbar */
  .tweaks-container::-webkit-scrollbar {
    width: 6px;
  }

  .tweaks-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .tweaks-container::-webkit-scrollbar-thumb {
    background: hsl(var(--border));
    border-radius: 3px;
  }

  .tweaks-container::-webkit-scrollbar-thumb:hover {
    background: hsl(var(--foreground-muted));
  }
</style>
