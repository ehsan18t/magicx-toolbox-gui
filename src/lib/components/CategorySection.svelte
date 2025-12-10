<script lang="ts">
  import type { TweakCategory, TweakWithStatus } from "$lib/types";
  import { CATEGORY_INFO } from "$lib/types";
  import Icon from "@iconify/svelte";
  import TweakCard from "./TweakCard.svelte";

  const {
    category,
    tweaks,
    expanded = true,
  } = $props<{
    category: TweakCategory;
    tweaks: TweakWithStatus[];
    expanded?: boolean;
  }>();

  let isExpanded = $state(expanded);

  // Use $derived for reactive category info
  const categoryInfo = $derived(CATEGORY_INFO[category as TweakCategory]);

  $effect(() => {
    isExpanded = expanded;
  });

  const appliedCount = $derived(tweaks.filter((t: TweakWithStatus) => t.status.is_applied).length);
</script>

<section class="category-section">
  <button class="category-header" onclick={() => (isExpanded = !isExpanded)}>
    <div class="header-left">
      <span class="category-icon">{categoryInfo.icon}</span>
      <div class="category-info">
        <h2 class="category-name">{categoryInfo.name}</h2>
        <p class="category-description">{categoryInfo.description}</p>
      </div>
    </div>
    <div class="header-right">
      <span class="tweak-count">
        {appliedCount}/{tweaks.length} applied
      </span>
      <Icon
        icon={isExpanded ? "mdi:chevron-up" : "mdi:chevron-down"}
        width="20"
        class="expand-icon"
      />
    </div>
  </button>

  {#if isExpanded}
    <div class="tweaks-grid">
      {#each tweaks as tweak (tweak.definition.id)}
        <TweakCard {tweak} />
      {/each}
    </div>
  {/if}
</section>

<style>
  .category-section {
    margin-bottom: 16px;
  }

  .category-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 12px 16px;
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    background: hsl(var(--card));
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .category-header:hover {
    border-color: hsl(var(--primary) / 0.3);
    background: hsl(var(--accent) / 0.5);
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

  .tweak-count {
    font-size: 12px;
    font-weight: 500;
    color: hsl(var(--muted-foreground));
    background: hsl(var(--muted));
    padding: 4px 10px;
    border-radius: 12px;
  }

  :global(.expand-icon) {
    color: hsl(var(--muted-foreground));
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
