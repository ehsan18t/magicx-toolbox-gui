<script lang="ts">
  import { activeTab, allTabs, type TabDefinition } from "$lib/stores/navigation";
  import { categoryStats, tweakStats } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import ColorSchemePicker from "./ColorSchemePicker.svelte";

  const SIDEBAR_PIN_KEY = "magicx-sidebar-pinned";

  let isExpanded = $state(false);
  let isPinned = $state(false);

  const sidebarExpanded = $derived(isExpanded || isPinned);

  // Load pin state from localStorage on mount
  onMount(() => {
    const savedPinState = localStorage.getItem(SIDEBAR_PIN_KEY);
    if (savedPinState === "true") {
      isPinned = true;
    }
  });

  function handleNavClick(tab: TabDefinition) {
    activeTab.set(tab.id);
  }

  function togglePin() {
    isPinned = !isPinned;
    localStorage.setItem(SIDEBAR_PIN_KEY, isPinned.toString());
    if (!isPinned) {
      isExpanded = false;
    }
  }

  function handleMouseEnter() {
    if (!isPinned) {
      isExpanded = true;
    }
  }

  function handleMouseLeave() {
    if (!isPinned) {
      isExpanded = false;
    }
  }
</script>

<aside
  class="relative z-100 flex h-full shrink-0 flex-col overflow-hidden border-r border-border bg-surface transition-[width] duration-250 ease-out {sidebarExpanded
    ? 'w-60'
    : 'w-16'}"
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  <!-- Logo / Brand -->
  <div class="flex min-h-16 items-center gap-3 border-b border-border p-4">
    <div class="flex h-8 w-8 shrink-0 items-center justify-center text-accent">
      <Icon icon="mdi:magic-staff" width="28" />
    </div>
    <span
      class="text-lg font-bold whitespace-nowrap text-foreground transition-all duration-200 {sidebarExpanded
        ? 'translate-x-0 opacity-100'
        : '-translate-x-2.5 opacity-0'}"
    >
      MagicX
    </span>
  </div>

  <!-- Navigation -->
  <nav class="flex flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto p-2">
    {#each $allTabs as tab (tab.id)}
      {@const stats = tab.id !== "overview" ? $categoryStats[tab.id] : null}
      {@const isActive = $activeTab === tab.id}
      <button
        class="group relative flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border-0 bg-transparent px-3 py-2.5 transition-all duration-150 {isActive
          ? 'bg-accent/15'
          : 'hover:bg-[hsl(var(--muted))]'}"
        onclick={() => handleNavClick(tab)}
        title={!sidebarExpanded ? tab.name : undefined}
      >
        <div
          class="relative flex h-6 w-6 shrink-0 items-center justify-center transition-colors duration-150 {isActive
            ? 'text-accent'
            : 'text-foreground-muted group-hover:text-accent'}"
        >
          <Icon icon={tab.icon || "mdi:folder"} width="22" />
          {#if stats && stats.applied > 0 && !sidebarExpanded}
            <span
              class="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full border-2 border-surface bg-success"
            ></span>
          {/if}
        </div>
        <span
          class="flex-1 text-left text-sm font-medium whitespace-nowrap transition-all duration-200 {isActive
            ? 'text-accent'
            : 'text-foreground'} {sidebarExpanded
            ? 'translate-x-0 opacity-100'
            : '-translate-x-2.5 opacity-0'}"
        >
          {tab.name}
        </span>
        {#if stats}
          <span
            class="rounded-full px-2 py-0.5 text-xs font-semibold transition-all duration-200 {sidebarExpanded
              ? 'translate-x-0 opacity-100'
              : '-translate-x-2.5 opacity-0'} {stats.applied === stats.total && stats.total > 0
              ? 'bg-success/15 text-success'
              : 'bg-[hsl(var(--muted))] text-foreground-muted'}"
          >
            {stats.applied}/{stats.total}
          </span>
        {/if}
        {#if isActive}
          <div
            class="absolute top-1/2 left-0 h-5 w-0.75 -translate-y-1/2 rounded-r-sm bg-accent"
          ></div>
        {/if}
      </button>
    {/each}
  </nav>

  <!-- Sidebar Footer -->
  <div class="flex flex-col gap-3 border-t border-border p-3">
    {#if sidebarExpanded}
      <div class="flex items-center justify-center gap-4 py-2">
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-lg font-bold text-foreground">{$tweakStats.applied}</span>
          <span class="text-[10px] font-medium tracking-wide text-foreground-muted uppercase"
            >Applied</span
          >
        </div>
        <div class="h-6 w-px bg-border"></div>
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-lg font-bold text-foreground">{$tweakStats.total}</span>
          <span class="text-[10px] font-medium tracking-wide text-foreground-muted uppercase"
            >Total</span
          >
        </div>
      </div>
    {/if}

    <!-- Color Scheme Picker -->
    <div
      class="flex items-center justify-center transition-all duration-200 {sidebarExpanded
        ? 'gap-2 py-1'
        : 'py-1'}"
      title="Color Scheme"
    >
      <ColorSchemePicker />
    </div>

    <!-- Pin toggle button -->
    <button
      class="flex w-full cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-2 transition-all duration-150 hover:bg-[hsl(var(--muted))] hover:text-foreground {isPinned
        ? 'text-accent'
        : 'text-foreground-muted'}"
      onclick={togglePin}
      title={isPinned ? "Unpin sidebar" : "Pin sidebar"}
    >
      <Icon icon={isPinned ? "mdi:pin" : "mdi:pin-outline"} width="18" />
    </button>
  </div>
</aside>
