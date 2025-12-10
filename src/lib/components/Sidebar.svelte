<script lang="ts">
  import { activeTab, allTabs, type TabDefinition } from "$lib/stores/navigation";
  import { categoryStats, tweakStats } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";

  let isExpanded = $state(false);
  let isPinned = $state(false);

  function handleNavClick(tab: TabDefinition) {
    activeTab.set(tab.id);
  }

  function togglePin() {
    isPinned = !isPinned;
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
  class="sidebar"
  class:expanded={isExpanded || isPinned}
  class:pinned={isPinned}
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  <!-- Logo / Brand -->
  <div class="sidebar-header">
    <div class="logo">
      <Icon icon="mdi:magic-staff" width="28" />
    </div>
    <span class="brand-text">MagicX</span>
  </div>

  <!-- Navigation -->
  <nav class="sidebar-nav">
    {#each $allTabs as tab (tab.id)}
      {@const stats = tab.id !== "overview" ? $categoryStats[tab.id] : null}
      {@const isActive = $activeTab === tab.id}
      <button
        class="nav-item"
        class:active={isActive}
        onclick={() => handleNavClick(tab)}
        title={!isExpanded && !isPinned ? tab.name : undefined}
      >
        <div class="nav-icon">
          <Icon icon={tab.icon || "mdi:folder"} width="22" />
          {#if stats && stats.applied > 0 && !isExpanded && !isPinned}
            <span class="mini-badge"></span>
          {/if}
        </div>
        <span class="nav-text">{tab.name}</span>
        {#if stats}
          <span class="nav-badge" class:all-done={stats.applied === stats.total && stats.total > 0}>
            {stats.applied}/{stats.total}
          </span>
        {/if}
        {#if isActive}
          <div class="active-indicator"></div>
        {/if}
      </button>
    {/each}
  </nav>

  <!-- Sidebar Footer -->
  <div class="sidebar-footer">
    <!-- Stats summary when expanded -->
    {#if isExpanded || isPinned}
      <div class="stats-summary">
        <div class="stat-item">
          <span class="stat-value">{$tweakStats.applied}</span>
          <span class="stat-label">Applied</span>
        </div>
        <div class="stat-divider"></div>
        <div class="stat-item">
          <span class="stat-value">{$tweakStats.total}</span>
          <span class="stat-label">Total</span>
        </div>
      </div>
    {/if}

    <!-- Pin toggle button -->
    <button class="pin-btn" onclick={togglePin} title={isPinned ? "Unpin sidebar" : "Pin sidebar"}>
      <Icon icon={isPinned ? "mdi:pin" : "mdi:pin-outline"} width="18" />
    </button>
  </div>
</aside>

<style>
  .sidebar {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 64px;
    height: 100%;
    background: hsl(var(--surface));
    border-right: 1px solid hsl(var(--border));
    transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
    z-index: 100;
    flex-shrink: 0;
  }

  .sidebar.expanded {
    width: 240px;
  }

  .sidebar.pinned {
    width: 240px;
  }

  /* Header */
  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    border-bottom: 1px solid hsl(var(--border));
    min-height: 64px;
  }

  .logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    color: hsl(var(--accent));
  }

  .brand-text {
    font-size: 18px;
    font-weight: 700;
    color: hsl(var(--foreground));
    white-space: nowrap;
    opacity: 0;
    transform: translateX(-10px);
    transition: all 0.2s ease;
  }

  .sidebar.expanded .brand-text,
  .sidebar.pinned .brand-text {
    opacity: 1;
    transform: translateX(0);
  }

  /* Navigation */
  .sidebar-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 8px;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border: none;
    background: transparent;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.15s ease;
    min-height: 44px;
  }

  .nav-item:hover {
    background: hsl(var(--muted));
  }

  .nav-item.active {
    background: hsl(var(--accent) / 0.15);
  }

  .nav-icon {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    color: hsl(var(--foreground-muted));
    transition: color 0.15s ease;
  }

  .nav-item:hover .nav-icon,
  .nav-item.active .nav-icon {
    color: hsl(var(--accent));
  }

  .mini-badge {
    position: absolute;
    top: -2px;
    right: -2px;
    width: 8px;
    height: 8px;
    background: hsl(var(--success));
    border-radius: 50%;
    border: 2px solid hsl(var(--surface));
  }

  .nav-text {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: hsl(var(--foreground));
    white-space: nowrap;
    opacity: 0;
    transform: translateX(-10px);
    transition: all 0.2s ease;
    text-align: left;
  }

  .sidebar.expanded .nav-text,
  .sidebar.pinned .nav-text {
    opacity: 1;
    transform: translateX(0);
  }

  .nav-item.active .nav-text {
    color: hsl(var(--accent));
  }

  .nav-badge {
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 600;
    border-radius: 12px;
    background: hsl(var(--muted));
    color: hsl(var(--foreground-muted));
    opacity: 0;
    transform: translateX(-10px);
    transition: all 0.2s ease;
  }

  .sidebar.expanded .nav-badge,
  .sidebar.pinned .nav-badge {
    opacity: 1;
    transform: translateX(0);
  }

  .nav-badge.all-done {
    background: hsl(var(--success) / 0.15);
    color: hsl(var(--success));
  }

  .active-indicator {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 20px;
    background: hsl(var(--accent));
    border-radius: 0 3px 3px 0;
  }

  /* Footer */
  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    border-top: 1px solid hsl(var(--border));
  }

  .stats-summary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 8px 0;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    color: hsl(var(--foreground));
  }

  .stat-label {
    font-size: 10px;
    font-weight: 500;
    color: hsl(var(--foreground-muted));
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-divider {
    width: 1px;
    height: 24px;
    background: hsl(var(--border));
  }

  .pin-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    padding: 8px;
    border: none;
    background: transparent;
    color: hsl(var(--foreground-muted));
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .pin-btn:hover {
    background: hsl(var(--muted));
    color: hsl(var(--foreground));
  }

  .sidebar.pinned .pin-btn {
    color: hsl(var(--accent));
  }

  /* Scrollbar for nav */
  .sidebar-nav::-webkit-scrollbar {
    width: 4px;
  }

  .sidebar-nav::-webkit-scrollbar-track {
    background: transparent;
  }

  .sidebar-nav::-webkit-scrollbar-thumb {
    background: hsl(var(--border));
    border-radius: 2px;
  }
</style>
