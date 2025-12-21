<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ColorSchemePicker } from "$lib/components/settings";
  import { Icon } from "$lib/components/shared";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { sidebarStore } from "$lib/stores/layout.svelte";
  import { openAboutModal, openSettingsModal, openUpdateModal } from "$lib/stores/modal.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { categoriesStore, getCategoryStats, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { slide } from "svelte/transition";

  // Derived values from stores
  const fixedTabs = $derived(navigationStore.fixedTabs);
  const categoryTabs = $derived(navigationStore.categoryTabs);
  const activeTab = $derived(navigationStore.activeTab);
  const stats = $derived(tweaksStore.stats);
  const categoryStats = $derived(getCategoryStats());
  const isUpdateAvailable = $derived(updateStore.isAvailable);
  const isCategoriesLoading = $derived(categoriesStore.isLoading);

  // Count tweaks with snapshots for badge
  const snapshotCount = $derived(tweaksStore.list.filter((t) => t.status.has_backup).length);

  // Count favorites for badge
  const favoritesCount = $derived(favoritesStore.count);

  // State for collapsible widgets section
  const isWidgetsOpen = $derived(sidebarStore.isWidgetsOpen);

  function handleNavClick(tab: TabDefinition) {
    navigationStore.navigateToTab(tab.id);
  }

  function togglePin() {
    sidebarStore.togglePinned();
  }

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
  <!-- Navigation -->
  <nav class="nav-scrollbar flex flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto p-2 pt-3">
    <!-- Fixed Tabs (Overview, Search, Favorites, Snapshots) -->
    {#each fixedTabs as tab (tab.id)}
      {@const isActive = activeTab === tab.id}
      {@const isSnapshots = tab.id === "snapshots"}
      {@const isFavorites = tab.id === "favorites"}
      {@const badgeCount = isSnapshots ? snapshotCount : isFavorites ? favoritesCount : 0}
      {@const badgeColor = isFavorites ? "bg-warning" : "bg-accent"}
      {@const badgeTextColor = isFavorites ? "text-warning" : "text-accent"}
      {@const badgeBgColor = isFavorites ? "bg-warning/15" : "bg-accent/15"}
      <button
        class="group relative flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border-0 bg-transparent px-3 py-2.5 transition-all duration-150 {isActive
          ? 'bg-accent/15'
          : 'hover:bg-muted'}"
        onclick={() => handleNavClick(tab)}
        use:tooltip={!sidebarStore.isOpen ? (badgeCount > 0 ? `${tab.name} (${badgeCount})` : tab.name) : null}
      >
        <div
          class="relative flex h-6 w-6 shrink-0 items-center justify-center transition-colors duration-150 {isActive
            ? isFavorites
              ? 'text-warning'
              : 'text-accent'
            : 'text-foreground-muted group-hover:text-accent'}"
        >
          <Icon icon={tab.icon || "mdi:folder"} width="22" />
          {#if badgeCount > 0 && !sidebarStore.isOpen}
            <span
              class="absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full border-2 border-surface px-0.5 text-[9px] font-bold text-white {badgeColor}"
            >
              {badgeCount > 99 ? "99+" : badgeCount}
            </span>
          {/if}
        </div>
        <span
          class="flex-1 text-left text-sm font-medium whitespace-nowrap transition-all duration-200 {isActive
            ? isFavorites
              ? 'text-warning'
              : 'text-accent'
            : 'text-foreground'} {sidebarStore.isOpen ? 'translate-x-0 opacity-100' : '-translate-x-2.5 opacity-0'}"
        >
          {tab.name}
        </span>
        {#if badgeCount > 0}
          <span
            class="rounded-full px-2 py-0.5 text-xs font-semibold transition-all duration-200 {badgeBgColor} {badgeTextColor} {sidebarStore.isOpen
              ? 'translate-x-0 opacity-100'
              : '-translate-x-2.5 opacity-0'}"
          >
            {badgeCount}
          </span>
        {/if}
        {#if isActive}
          <div
            class="absolute top-1/2 left-0 h-5 w-0.75 -translate-y-1/2 rounded-r-sm {isFavorites
              ? 'bg-warning'
              : 'bg-accent'}"
          ></div>
        {/if}
      </button>
    {/each}

    <!-- Divider -->
    <div class="border-b border-border"></div>

    <!-- Categories Header -->
    {#if sidebarStore.isOpen}
      <div class="px-3 py-2">
        <span class="text-xs font-semibold tracking-wider text-foreground-muted uppercase">Categories</span>
      </div>
    {:else}
      <div class="flex justify-center py-2">
        <div class="h-1 w-1 rounded-full bg-foreground-muted"></div>
      </div>
    {/if}

    <!-- Category Tabs -->
    {#each categoryTabs as tab (tab.id)}
      {@const tabStats = tab.id !== "overview" ? categoryStats[tab.id] : null}
      {@const isActive = activeTab === tab.id}
      <button
        class="group relative flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border-0 bg-transparent px-3 py-2.5 transition-all duration-150 {isActive
          ? 'bg-accent/15'
          : 'hover:bg-muted'}"
        onclick={() => handleNavClick(tab)}
        use:tooltip={!sidebarStore.isOpen ? tab.name : null}
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
      {#if isWidgetsOpen}
        <div transition:slide={{ duration: 200, axis: "y" }}>
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
          <div class="flex items-center justify-center gap-2 py-1">
            <ColorSchemePicker />
          </div>
        </div>
      {/if}
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
        aria-label={sidebarStore.isPinned ? "Unpin sidebar" : "Pin sidebar"}
        aria-pressed={sidebarStore.isPinned}
        class="{sidebarStore.isPinned ? 'text-accent' : 'text-foreground-muted'}
        {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
        onclick={togglePin}
        use:tooltip={sidebarStore.isPinned ? "Unpin sidebar" : "Pin sidebar"}
      >
        <Icon icon={sidebarStore.isPinned ? "mdi:pin" : "mdi:pin-outline"} width="22" />
      </button>

      <!-- Widgets toggle button -->
      <button
        type="button"
        aria-label={isWidgetsOpen ? "Hide widgets" : "Show widgets"}
        aria-pressed={isWidgetsOpen}
        class="{isWidgetsOpen ? 'text-accent' : 'text-foreground-muted'}
        {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
        onclick={() => sidebarStore.toggleWidgets()}
        use:tooltip={isWidgetsOpen ? "Hide widgets" : "Show widgets"}
      >
        <Icon icon="mdi:widgets" width="22" />
      </button>

      <!-- Update button -->
      <button
        type="button"
        aria-label={isUpdateAvailable ? "Update available" : "Updates"}
        class="relative {isUpdateAvailable ? 'text-success' : 'text-foreground-muted'} {sidebarStore.isOpen
          ? 'shrink-0'
          : 'w-full'}"
        onclick={openUpdateModal}
        use:tooltip={isUpdateAvailable ? "Update available!" : "Updates"}
      >
        <Icon icon="mdi:update" width="22" />
        {#if isUpdateAvailable}
          <span class="absolute -top-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-surface bg-success"></span>
        {/if}
      </button>

      <!-- Settings button -->
      <button
        type="button"
        aria-label="Settings"
        class="text-foreground-muted {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
        onclick={openSettingsModal}
        use:tooltip={"Settings"}
      >
        <Icon icon="mdi:settings-outline" width="22" />
      </button>

      <!-- About button -->
      <button
        type="button"
        aria-label="About"
        class="text-foreground-muted {sidebarStore.isOpen ? 'shrink-0' : 'w-full'}"
        onclick={openAboutModal}
        use:tooltip={"About"}
      >
        <Icon icon="mdi:information-outline" width="22" />
      </button>
    </div>
  </div>
</aside>

<style lang="postcss">
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
