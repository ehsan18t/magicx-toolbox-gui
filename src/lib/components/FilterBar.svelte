<script lang="ts">
  import { categoriesStore, searchQuery, selectedCategory } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";
</script>

<div class="filter-bar">
  <div class="search-box">
    <Icon icon="mdi:magnify" width="18" class="search-icon" />
    <input type="text" placeholder="Search tweaks..." bind:value={$searchQuery} />
    {#if $searchQuery}
      <button class="clear-btn" onclick={() => ($searchQuery = "")}>
        <Icon icon="mdi:close" width="16" />
      </button>
    {/if}
  </div>

  <div class="category-filters">
    <button
      class="filter-btn"
      class:active={$selectedCategory === "all"}
      onclick={() => ($selectedCategory = "all")}
    >
      All
    </button>
    {#each $categoriesStore as cat (cat.id)}
      <button
        class="filter-btn"
        class:active={$selectedCategory === cat.id}
        onclick={() => ($selectedCategory = cat.id)}
      >
        <span class="cat-icon">{cat.icon}</span>
        <span class="cat-name">{cat.name}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .filter-bar {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }

  :global(.search-icon) {
    position: absolute;
    left: 12px;
    color: hsl(var(--muted-foreground));
    pointer-events: none;
  }

  .search-box input {
    width: 100%;
    padding: 10px 36px 10px 40px;
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    background: hsl(var(--background));
    color: hsl(var(--foreground));
    font-size: 14px;
    transition: border-color 0.2s ease;
  }

  .search-box input:focus {
    outline: none;
    border-color: hsl(var(--primary));
  }

  .search-box input::placeholder {
    color: hsl(var(--muted-foreground));
  }

  .clear-btn {
    position: absolute;
    right: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .clear-btn:hover {
    background: hsl(var(--muted));
    color: hsl(var(--foreground));
  }

  .category-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .filter-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid hsl(var(--border));
    border-radius: 20px;
    background: hsl(var(--background));
    color: hsl(var(--muted-foreground));
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-btn:hover {
    border-color: hsl(var(--primary) / 0.5);
    color: hsl(var(--foreground));
  }

  .filter-btn.active {
    background: hsl(var(--primary));
    border-color: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
  }

  .cat-icon {
    font-size: 14px;
  }

  .cat-name {
    display: none;
  }

  @media (min-width: 640px) {
    .cat-name {
      display: inline;
    }
  }

  @media (min-width: 768px) {
    .filter-bar {
      flex-direction: row;
      align-items: center;
      justify-content: space-between;
    }

    .search-box {
      flex: 0 0 300px;
    }
  }
</style>
