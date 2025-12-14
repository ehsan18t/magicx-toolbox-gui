<script lang="ts">
  import { sidebarStore } from "$lib/stores/layout.svelte";
  import { openAboutModal, openSettingsModal, openUpdateModal } from "$lib/stores/modal.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { categoriesStore, getCategoryStats, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { onMount } from "svelte";
  import ColorSchemePicker from "./ColorSchemePicker.svelte";
  import Icon from "./Icon.svelte";

  const SIDEBAR_PIN_KEY = "magicx-sidebar-pinned";

  // Derived values from stores
  const allTabs = $derived(navigationStore.allTabs);
  const activeTab = $derived(navigationStore.activeTab);
  const stats = $derived(tweaksStore.stats);
  const categoryStats = $derived(getCategoryStats());
  const isUpdateAvailable = $derived(updateStore.isAvailable);
  const isCategoriesLoading = $derived(categoriesStore.isLoading);

  // Guard to prevent effect from saving before mount loads the saved value
  let pinStateInitialized = false;

  // Load pin state from localStorage on mount
  onMount(() => {
    const savedPinState = localStorage.getItem(SIDEBAR_PIN_KEY);
    if (savedPinState === "true") {
      sidebarStore.init(true);
    }
    pinStateInitialized = true;
  });

  function handleNavClick(tab: TabDefinition) {
    navigationStore.navigateToTab(tab.id);
  }

  function togglePin() {
    sidebarStore.togglePinned();
  }

  // Effect to save pin state (only after initial load to prevent race condition)
  $effect(() => {
    if (pinStateInitialized) {
      localStorage.setItem(SIDEBAR_PIN_KEY, sidebarStore.isPinned.toString());
    }
  });

  function handleMouseEnter() {
    if (!sidebarStore.isPinned) {
      sidebarStore.setExpanded(true);
    }
  }

  function handleMouseLeave() {
    if (!sidebarStore.isPinned) {
      sidebarStore.setExpanded(false);
    }
  }
</script>

<aside
  class="relative z-100 flex h-full shrink-0 flex-col overflow-hidden border-r border-border bg-surface transition-[width] duration-250 ease-out {sidebarStore.isOpen
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
      class="text-lg font-bold whitespace-nowrap text-foreground transition-all duration-200 {sidebarStore.isOpen
        ? 'translate-x-0 opacity-100'
        : '-translate-x-2.5 opacity-0'}"
    >
      MagicX
    </span>
  </div>

  <!-- Navigation -->
  <nav class="nav-scrollbar flex flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto p-2">
    {#each allTabs as tab (tab.id)}
      {@const tabStats = tab.id !== "overview" ? categoryStats[tab.id] : null}
      {@const isActive = activeTab === tab.id}
      <button
        class="group relative flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border-0 bg-transparent px-3 py-2.5 transition-all duration-150 {isActive
          ? 'bg-accent/15'
          : 'hover:bg-muted'}"
        onclick={() => handleNavClick(tab)}
        title={!sidebarStore.isOpen ? tab.name : undefined}
      >
        <div
          class="relative flex h-6 w-6 shrink-0 items-center justify-center transition-colors duration-150 {isActive
            ? 'text-accent'
            : 'text-foreground-muted group-hover:text-accent'}"
        >
          <Icon icon={tab.icon || "mdi:folder"} width="22" />
          {#if tabStats && tabStats.applied > 0 && !sidebarStore.isOpen}
            <span class="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full border-2 border-surface bg-success"></span>
          {/if}
        </div>
        <span
          class="flex-1 text-left text-sm font-medium whitespace-nowrap transition-all duration-200 {isActive
            ? 'text-accent'
            : 'text-foreground'} {sidebarStore.isOpen ? 'translate-x-0 opacity-100' : '-translate-x-2.5 opacity-0'}"
        >
          {tab.name}
        </span>
        {#if tabStats}
          <span
            class="rounded-full px-2 py-0.5 text-xs font-semibold transition-all duration-200 {sidebarStore.isOpen
              ? 'translate-x-0 opacity-100'
              : '-translate-x-2.5 opacity-0'} {tabStats.applied === tabStats.total && tabStats.total > 0
              ? 'bg-success/15 text-success'
              : 'bg-muted text-foreground-muted'}"
          >
            {tabStats.applied}/{tabStats.total}
          </span>
        {/if}
        {#if isActive}
          <div class="absolute top-1/2 left-0 h-5 w-0.75 -translate-y-1/2 rounded-r-sm bg-accent"></div>
        {/if}
      </button>
    {/each}

    <!-- Skeleton loading for category tabs while loading -->
    {#if isCategoriesLoading}
      {#each [0, 1, 2, 3, 4, 5] as i (`nav-skeleton-${i}`)}
        <div class="flex min-h-11 items-center gap-3 px-3 py-2.5">
          <div class="animate-pulse bg-muted h-6 w-6 shrink-0 rounded"></div>
          {#if sidebarStore.isOpen}
            <div class="animate-pulse bg-muted h-4 flex-1 rounded"></div>
          {/if}
        </div>
      {/each}
    {/if}
  </nav>

  <!-- Sidebar Footer -->
  <div class="flex flex-col gap-3 border-t border-border p-3">
    {#if sidebarStore.isOpen}
      <div class="flex items-center justify-center gap-4 py-2">
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-lg font-bold text-foreground">{stats.applied}</span>
          <span class="text-[10px] font-medium tracking-wide text-foreground-muted uppercase">Applied</span>
        </div>
        <div class="h-6 w-px bg-border"></div>
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-lg font-bold text-foreground">{stats.total}</span>
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
      class="sidebar-controls flex items-center gap-2 transition-all duration-200 {sidebarStore.isOpen
        ? 'flex-row-reverse justify-center'
        : 'flex-col justify-center'}"
    >
      <!-- Pin toggle button -->
      <button
        type="button"
        aria-pressed={sidebarStore.isPinned}
        class="{sidebarStore.isPinned ? 'text-accent' : 'text-foreground-muted'}
        {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
        onclick={togglePin}
        title={sidebarStore.isPinned ? "Unpin sidebar" : "Pin sidebar"}
      >
        <Icon icon={sidebarStore.isPinned ? "mdi:pin" : "mdi:pin-outline"} width="22" />
      </button>

      <!-- Update button -->
      <button
        type="button"
        class="relative {isUpdateAvailable ? 'text-success' : 'text-foreground-muted'} {sidebarStore.isOpen
          ? 'shrink-0'
          : 'w-full'}"
        onclick={openUpdateModal}
        title={isUpdateAvailable ? "Update available!" : "Updates"}
      >
        <Icon icon="mdi:update" width="22" />
        {#if isUpdateAvailable}
          <span class="absolute -top-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-surface bg-success"></span>
        {/if}
      </button>

      <!-- Settings button -->
      <button
        type="button"
        class="text-foreground-muted {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
        onclick={openSettingsModal}
        title="Settings"
      >
        <Icon icon="mdi:settings-outline" width="22" />
      </button>

      <!-- About button -->
      <button
        type="button"
        class="text-foreground-muted {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
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
