<script lang="ts">
  import { categoriesStore, searchQuery, selectedCategory } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";
</script>

<div class="mb-4 flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
  <!-- Search box -->
  <div class="relative flex items-center md:w-75 md:flex-none">
    <Icon
      icon="mdi:magnify"
      width="18"
      class="pointer-events-none absolute left-3 text-foreground-muted"
    />
    <input
      type="text"
      placeholder="Search tweaks..."
      bind:value={$searchQuery}
      class="w-full rounded-lg border border-border bg-background py-2.5 pr-9 pl-10 text-sm text-foreground transition-colors duration-200 placeholder:text-foreground-muted focus:border-primary focus:outline-none"
    />
    {#if $searchQuery}
      <button
        class="absolute right-2 flex h-6 w-6 cursor-pointer items-center justify-center rounded border-0 bg-transparent text-foreground-muted transition-all duration-200 hover:bg-[hsl(var(--muted))] hover:text-foreground"
        onclick={() => ($searchQuery = "")}
      >
        <Icon icon="mdi:close" width="16" />
      </button>
    {/if}
  </div>

  <!-- Category filters -->
  <div class="flex flex-wrap gap-2">
    <button
      class="flex cursor-pointer items-center gap-1.5 rounded-full border border-border bg-background px-3 py-1.5 text-sm text-foreground-muted transition-all duration-200 hover:border-primary/50 hover:text-foreground {$selectedCategory ===
      'all'
        ? 'border-primary bg-primary text-primary-foreground'
        : ''}"
      onclick={() => ($selectedCategory = "all")}
    >
      All
    </button>
    {#each $categoriesStore as cat (cat.id)}
      <button
        class="flex cursor-pointer items-center gap-1.5 rounded-full border border-border bg-background px-3 py-1.5 text-sm text-foreground-muted transition-all duration-200 hover:border-primary/50 hover:text-foreground {$selectedCategory ===
        cat.id
          ? 'border-primary bg-primary text-primary-foreground'
          : ''}"
        onclick={() => ($selectedCategory = cat.id)}
      >
        <span class="text-sm">{cat.icon}</span>
        <span class="hidden sm:inline">{cat.name}</span>
      </button>
    {/each}
  </div>
</div>
