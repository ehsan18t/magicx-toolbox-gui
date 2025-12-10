<script lang="ts">
  import { applyTweak, loadingStore, revertTweak } from "$lib/stores/tweaks";
  import type { TweakCategory, TweakWithStatus } from "$lib/types";
  import { CATEGORY_INFO } from "$lib/types";
  import Icon from "@iconify/svelte";
  import { derived } from "svelte/store";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import TweakCard from "./TweakCard.svelte";

  interface Props {
    category: TweakCategory;
    tweaks: TweakWithStatus[];
    initialExpanded?: boolean;
  }

  const { category, tweaks, initialExpanded = true }: Props = $props();

  // Use a function to initialize state to avoid state_referenced_locally warning
  const getInitialExpanded = () => initialExpanded;
  let isExpanded = $state(getInitialExpanded());
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  const categoryInfo = $derived(CATEGORY_INFO[category]);
  const appliedCount = $derived(tweaks.filter((t) => t.status.is_applied).length);
  const unappliedTweaks = $derived(tweaks.filter((t) => !t.status.is_applied));
  const appliedTweaks = $derived(tweaks.filter((t) => t.status.is_applied));

  // Check if any tweak in this category is loading
  const isLoading = derived(loadingStore, ($loading) =>
    tweaks.some((t) => $loading.has(t.definition.id)),
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

  function onApplyAllClick(e: MouseEvent) {
    e.stopPropagation();
    if (unappliedTweaks.length > 0) {
      showApplyAllDialog = true;
    }
  }

  function onRevertAllClick(e: MouseEvent) {
    e.stopPropagation();
    if (appliedTweaks.length > 0) {
      showRevertAllDialog = true;
    }
  }
</script>

<section class="category-section">
  <div class="category-header">
    <button class="header-toggle" onclick={() => (isExpanded = !isExpanded)}>
      <div class="header-left">
        <span class="category-icon">{categoryInfo.icon}</span>
        <div class="category-info">
          <h2 class="category-name">{categoryInfo.name}</h2>
          <p class="category-description">{categoryInfo.description}</p>
        </div>
      </div>
    </button>
    <div class="header-right">
      <div class="batch-actions">
        <button
          class="batch-btn apply"
          onclick={onApplyAllClick}
          disabled={unappliedTweaks.length === 0 || $isLoading || isBatchProcessing}
          title="Apply all unapplied tweaks in this category"
        >
          {#if isBatchProcessing}
            <Icon icon="mdi:loading" width="14" class="spin" />
          {:else}
            <Icon icon="mdi:check-all" width="14" />
          {/if}
          Apply All
        </button>
        <button
          class="batch-btn revert"
          onclick={onRevertAllClick}
          disabled={appliedTweaks.length === 0 || $isLoading || isBatchProcessing}
          title="Revert all applied tweaks in this category"
        >
          {#if isBatchProcessing}
            <Icon icon="mdi:loading" width="14" class="spin" />
          {:else}
            <Icon icon="mdi:undo-variant" width="14" />
          {/if}
          Revert All
        </button>
      </div>
      <span class="tweak-count">
        {appliedCount}/{tweaks.length} applied
      </span>
      <button class="expand-btn" onclick={() => (isExpanded = !isExpanded)}>
        <Icon
          icon={isExpanded ? "mdi:chevron-up" : "mdi:chevron-down"}
          width="20"
          class="expand-icon"
        />
      </button>
    </div>
  </div>

  {#if isExpanded}
    <div class="tweaks-grid">
      {#each tweaks as tweak (tweak.definition.id)}
        <TweakCard {tweak} />
      {/each}
    </div>
  {/if}
</section>

<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply All {categoryInfo.name} Tweaks"
  message="This will apply {unappliedTweaks.length} tweak{unappliedTweaks.length === 1
    ? ''
    : 's'} in the {categoryInfo.name} category. Some tweaks may require administrator privileges or a system restart."
  confirmText="Apply All"
  variant="warning"
  onconfirm={handleApplyAll}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Revert All {categoryInfo.name} Tweaks"
  message="This will revert {appliedTweaks.length} applied tweak{appliedTweaks.length === 1
    ? ''
    : 's'} in the {categoryInfo.name} category to their default values."
  confirmText="Revert All"
  variant="danger"
  onconfirm={handleRevertAll}
  oncancel={() => (showRevertAllDialog = false)}
/>

<style>
  .category-section {
    margin-bottom: 16px;
  }

  .category-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 0;
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    background: hsl(var(--card));
    transition: all 0.2s ease;
  }

  .category-header:hover {
    border-color: hsl(var(--primary) / 0.3);
    background: hsl(var(--accent) / 0.5);
  }

  .header-toggle {
    display: flex;
    align-items: center;
    flex: 1;
    padding: 12px 16px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .category-icon {
    font-size: 24px;
  }

  .category-info {
    text-align: left;
  }

  .category-name {
    font-size: 16px;
    font-weight: 600;
    color: hsl(var(--foreground));
    margin: 0;
  }

  .category-description {
    font-size: 12px;
    color: hsl(var(--muted-foreground));
    margin: 2px 0 0 0;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .batch-actions {
    display: flex;
    gap: 6px;
  }

  .batch-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    border-radius: 6px;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .batch-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .batch-btn.apply {
    background: hsl(142 76% 36% / 0.15);
    color: hsl(142 76% 36%);
    border-color: hsl(142 76% 36% / 0.3);
  }

  .batch-btn.apply:hover:not(:disabled) {
    background: hsl(142 76% 36% / 0.25);
  }

  .batch-btn.revert {
    background: hsl(0 84% 60% / 0.15);
    color: hsl(0 84% 60%);
    border-color: hsl(0 84% 60% / 0.3);
  }

  .batch-btn.revert:hover:not(:disabled) {
    background: hsl(0 84% 60% / 0.25);
  }

  :global(.batch-btn .spin) {
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

  .tweak-count {
    font-size: 12px;
    font-weight: 500;
    color: hsl(var(--muted-foreground));
    background: hsl(var(--muted));
    padding: 4px 10px;
    border-radius: 12px;
  }

  .expand-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px;
    margin-right: 8px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 4px;
    color: hsl(var(--muted-foreground));
    transition: all 0.15s ease;
  }

  .expand-btn:hover {
    background: hsl(var(--muted));
    color: hsl(var(--foreground));
  }

  :global(.expand-icon) {
    color: currentColor;
    transition: transform 0.2s ease;
  }

  .tweaks-grid {
    display: grid;
    gap: 8px;
    margin-top: 8px;
    padding-left: 16px;
  }

  @media (min-width: 768px) {
    .tweaks-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (min-width: 1200px) {
    .tweaks-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }
</style>
