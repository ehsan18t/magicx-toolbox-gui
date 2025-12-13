<script lang="ts">
  import { isSidebarOpen, sidebarState } from "$lib/stores/layout";
  import { openAboutModal, openSettingsModal, openUpdateModal } from "$lib/stores/modal";
  import { activeTab, allTabs, type TabDefinition } from "$lib/stores/navigation";
  import { categoryStats, tweakStats } from "$lib/stores/tweaks";
  import { isUpdateAvailable } from "$lib/stores/update";
  import { onMount } from "svelte";
  import ColorSchemePicker from "./ColorSchemePicker.svelte";
  import Icon from "./Icon.svelte";

  const SIDEBAR_PIN_KEY = "magicx-sidebar-pinned";

  // Load pin state from localStorage on mount
  onMount(() => {
    const savedPinState = localStorage.getItem(SIDEBAR_PIN_KEY);
    if (savedPinState === "true") {
      sidebarState.init(true);
    }
  });

  function handleNavClick(tab: TabDefinition) {
    activeTab.set(tab.id);
  }

  function togglePin() {
    sidebarState.togglePinned();
    // Subscribe to get current state (since we need to save to localstorage)
    // A bit hacky but works for simple case or we can just update localstorage in the store subscription if needed
    // For now, let's just cheat and check the store value 'next tick' or use derived
    // Actually simpler: just toggle and save the INVERSE of previous knowledge?
    // Let's rely on the store update.
    // Ideally we should sync localStorage inside the store or effect.
  }

  // Effect to save pin state
  $effect(() => {
    localStorage.setItem(SIDEBAR_PIN_KEY, $sidebarState.isPinned.toString());
  });

  function handleMouseEnter() {
    if (!$sidebarState.isPinned) {
      sidebarState.setExpanded(true);
    }
  }

  function handleMouseLeave() {
    if (!$sidebarState.isPinned) {
      sidebarState.setExpanded(false);
    }
  }
</script>

<aside
  class="relative z-100 flex h-full shrink-0 flex-col overflow-hidden border-r border-border bg-surface transition-[width] duration-250 ease-out {$isSidebarOpen
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
      class="text-lg font-bold whitespace-nowrap text-foreground transition-all duration-200 {$isSidebarOpen
        ? 'translate-x-0 opacity-100'
        : '-translate-x-2.5 opacity-0'}"
    >
      MagicX
    </span>
  </div>

  <!-- Navigation -->
  <nav class="nav-scrollbar flex flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto p-2">
    {#each $allTabs as tab (tab.id)}
      {@const stats = tab.id !== "overview" ? $categoryStats[tab.id] : null}
      {@const isActive = $activeTab === tab.id}
      <button
        class="group relative flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border-0 bg-transparent px-3 py-2.5 transition-all duration-150 {isActive
          ? 'bg-accent/15'
          : 'hover:bg-[hsl(var(--muted))]'}"
        onclick={() => handleNavClick(tab)}
        title={!$isSidebarOpen ? tab.name : undefined}
      >
        <div
          class="relative flex h-6 w-6 shrink-0 items-center justify-center transition-colors duration-150 {isActive
            ? 'text-accent'
            : 'text-foreground-muted group-hover:text-accent'}"
        >
          <Icon icon={tab.icon || "mdi:folder"} width="22" />
          {#if stats && stats.applied > 0 && !$isSidebarOpen}
            <span class="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full border-2 border-surface bg-success"></span>
          {/if}
        </div>
        <span
          class="flex-1 text-left text-sm font-medium whitespace-nowrap transition-all duration-200 {isActive
            ? 'text-accent'
            : 'text-foreground'} {$isSidebarOpen ? 'translate-x-0 opacity-100' : '-translate-x-2.5 opacity-0'}"
        >
          {tab.name}
        </span>
        {#if stats}
          <span
            class="rounded-full px-2 py-0.5 text-xs font-semibold transition-all duration-200 {$isSidebarOpen
              ? 'translate-x-0 opacity-100'
              : '-translate-x-2.5 opacity-0'} {stats.applied === stats.total && stats.total > 0
              ? 'bg-success/15 text-success'
              : 'bg-[hsl(var(--muted))] text-foreground-muted'}"
          >
            {stats.applied}/{stats.total}
          </span>
        {/if}
        {#if isActive}
          <div class="absolute top-1/2 left-0 h-5 w-0.75 -translate-y-1/2 rounded-r-sm bg-accent"></div>
        {/if}
      </button>
    {/each}
  </nav>

  <!-- Sidebar Footer -->
  <div class="flex flex-col gap-3 border-t border-border p-3">
    {#if $isSidebarOpen}
      <div class="flex items-center justify-center gap-4 py-2">
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-lg font-bold text-foreground">{$tweakStats.applied}</span>
          <span class="text-[10px] font-medium tracking-wide text-foreground-muted uppercase">Applied</span>
        </div>
        <div class="h-6 w-px bg-border"></div>
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-lg font-bold text-foreground">{$tweakStats.total}</span>
          <span class="text-[10px] font-medium tracking-wide text-foreground-muted uppercase">Total</span>
        </div>
      </div>

      <!-- Color Scheme Picker (only visible when expanded) -->
      <div class="flex items-center justify-center gap-2 py-1" title="Color Scheme">
        <ColorSchemePicker />
      </div>
    {/if}

    <!-- Control buttons: Pin, Update, Settings, About -->
    <div
      class="sidebar-controls flex items-center gap-2 transition-all duration-200 {$isSidebarOpen
        ? 'flex-row-reverse justify-center'
        : 'flex-col justify-center'}"
    >
      <!-- Pin toggle button -->
      <button
        class="{$sidebarState.isPinned ? 'text-accent' : 'text-foreground-muted'}
        {$isSidebarOpen ? 'shrink-0' : 'w-full'}"
        onclick={togglePin}
        title={$sidebarState.isPinned ? "Unpin sidebar" : "Pin sidebar"}
      >
        <Icon icon={$sidebarState.isPinned ? "mdi:pin" : "mdi:pin-outline"} width="22" />
      </button>

      <!-- Update button -->
      <button
        class="relative {$isUpdateAvailable ? 'text-success' : 'text-foreground-muted'} {$isSidebarOpen
          ? 'shrink-0'
          : 'w-full'}"
        onclick={openUpdateModal}
        title={$isUpdateAvailable ? "Update available!" : "Updates"}
      >
        <Icon icon="mdi:update" width="22" />
        {#if $isUpdateAvailable}
          <span class="absolute -top-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-surface bg-success"></span>
        {/if}
      </button>

      <!-- Settings button -->
      <button
        class="text-foreground-muted {$isSidebarOpen ? 'shrink-0' : 'w-full'}"
        onclick={openSettingsModal}
        title="Settings"
      >
        <Icon icon="mdi:settings-outline" width="22" />
      </button>

      <!-- About button -->
      <button
        class="text-foreground-muted {$isSidebarOpen ? 'shrink-0' : 'w-full'}"
        onclick={openAboutModal}
        title="About"
      >
        <Icon icon="mdi:information-outline" width="22" />
      </button>
    </div>
  </div>
</aside>

<style>
  @reference "@/app.css";

  .sidebar-controls {
    & > button {
      @apply flex cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-2 transition-all duration-150 hover:bg-accent/10 hover:text-accent;
    }
  }

  /* Hide scrollbar but keep scrolling */
  .nav-scrollbar {
    scrollbar-width: none; /* Firefox */
    -ms-overflow-style: none; /* IE/Edge */

    &::-webkit-scrollbar {
      display: none; /* Chrome/Safari/Opera */
    }
  }
</style>
