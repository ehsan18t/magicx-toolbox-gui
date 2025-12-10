<script lang="ts">
  import { navigateToCategory } from "$lib/stores/navigation";
  import {
    categoriesStore,
    categoryStats,
    pendingRebootCount,
    systemStore,
    tweakStats,
  } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";

  // Progress percentage
  const progressPercent = $derived(
    $tweakStats.total > 0 ? Math.round(($tweakStats.applied / $tweakStats.total) * 100) : 0,
  );
</script>

<div class="overview">
  <!-- Welcome Header -->
  <header class="welcome-header">
    <div class="welcome-text">
      <h1>Welcome back!</h1>
      <p>Manage your Windows tweaks and optimizations</p>
    </div>
    {#if $pendingRebootCount > 0}
      <div class="reboot-alert">
        <Icon icon="mdi:restart-alert" width="20" />
        <span>{$pendingRebootCount} pending reboot</span>
      </div>
    {/if}
  </header>

  <!-- Stats Cards Row -->
  <section class="stats-row">
    <!-- Progress Ring Card -->
    <div class="stat-card progress-card">
      <div class="progress-ring" style="--progress: {progressPercent}">
        <svg viewBox="0 0 100 100">
          <circle class="bg" cx="50" cy="50" r="40" />
          <circle class="progress" cx="50" cy="50" r="40" />
        </svg>
        <span class="progress-value">{progressPercent}%</span>
      </div>
      <div class="progress-label">
        <span class="label-main">Optimization</span>
        <span class="label-sub">Progress</span>
      </div>
    </div>

    <!-- Total Tweaks -->
    <div class="stat-card">
      <div class="stat-icon total">
        <Icon icon="mdi:tune-vertical" width="24" />
      </div>
      <div class="stat-content">
        <span class="stat-number">{$tweakStats.total}</span>
        <span class="stat-label">Total Tweaks</span>
      </div>
    </div>

    <!-- Applied Tweaks -->
    <div class="stat-card">
      <div class="stat-icon applied">
        <Icon icon="mdi:check-circle" width="24" />
      </div>
      <div class="stat-content">
        <span class="stat-number">{$tweakStats.applied}</span>
        <span class="stat-label">Applied</span>
      </div>
    </div>

    <!-- Pending Tweaks -->
    <div class="stat-card">
      <div class="stat-icon pending">
        <Icon icon="mdi:clock-outline" width="24" />
      </div>
      <div class="stat-content">
        <span class="stat-number">{$tweakStats.pending}</span>
        <span class="stat-label">Pending</span>
      </div>
    </div>
  </section>

  <!-- System Info Card -->
  <section class="system-card">
    <div class="system-header">
      <Icon icon="mdi:monitor" width="20" />
      <h2>System Information</h2>
    </div>
    <div class="system-grid">
      <div class="system-item">
        <Icon icon="mdi:microsoft-windows" width="18" />
        <div class="system-info">
          <span class="info-label">Operating System</span>
          <span class="info-value">{$systemStore?.windows?.product_name ?? "Windows"}</span>
        </div>
      </div>
      <div class="system-item">
        <Icon icon="mdi:tag" width="18" />
        <div class="system-info">
          <span class="info-label">Version</span>
          <span class="info-value"
            >{$systemStore?.windows?.display_version ?? ""} (Build {$systemStore?.windows
              ?.build_number ?? ""})</span
          >
        </div>
      </div>
      <div class="system-item">
        <Icon icon="mdi:account" width="18" />
        <div class="system-info">
          <span class="info-label">User</span>
          <span class="info-value"
            >{$systemStore?.username ?? ""}@{$systemStore?.computer_name ?? ""}</span
          >
        </div>
      </div>
      <div class="system-item">
        <Icon icon="mdi:shield-check" width="18" />
        <div class="system-info">
          <span class="info-label">Privileges</span>
          <span class="info-value privilege" class:admin={$systemStore?.is_admin}>
            {$systemStore?.is_admin ? "Administrator" : "Standard User"}
          </span>
        </div>
      </div>
    </div>
  </section>

  <!-- Categories Grid -->
  <section class="categories-section">
    <div class="section-header">
      <h2>Categories</h2>
      <span class="section-subtitle">{$categoriesStore.length} available</span>
    </div>
    <div class="categories-grid">
      {#each $categoriesStore as category (category.id)}
        {@const stats = $categoryStats[category.id]}
        {@const progress = stats?.total > 0 ? (stats.applied / stats.total) * 100 : 0}
        <button class="category-card" onclick={() => navigateToCategory(category.id)}>
          <div class="card-header">
            <div class="category-icon">
              <Icon icon={category.icon || "mdi:folder"} width="24" />
            </div>
            <div class="category-badge" class:complete={progress === 100 && stats?.total > 0}>
              {stats?.applied ?? 0}/{stats?.total ?? 0}
            </div>
          </div>
          <div class="card-content">
            <h3>{category.name}</h3>
            <p>{category.description}</p>
          </div>
          <div class="card-progress">
            <div class="progress-bar">
              <div class="progress-fill" style="width: {progress}%"></div>
            </div>
          </div>
          <div class="card-arrow">
            <Icon icon="mdi:arrow-right" width="18" />
          </div>
        </button>
      {/each}
    </div>
  </section>

  <!-- Footer Info -->
  <footer class="overview-footer">
    <div class="footer-tip">
      <Icon icon="mdi:lightbulb-outline" width="18" />
      <p>
        <strong>Tip:</strong> Changes are backed up automatically. Hover over the sidebar to expand it.
      </p>
    </div>
  </footer>
</div>

<style>
  .overview {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 1400px;
    margin: 0 auto;
    width: 100%;
  }

  /* Welcome Header */
  .welcome-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .welcome-text h1 {
    font-size: 28px;
    font-weight: 700;
    color: hsl(var(--foreground));
    margin: 0;
    letter-spacing: -0.5px;
  }

  .welcome-text p {
    font-size: 14px;
    color: hsl(var(--foreground-muted));
    margin: 4px 0 0 0;
  }

  .reboot-alert {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: hsl(var(--warning) / 0.15);
    color: hsl(var(--warning));
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
  }

  /* Stats Row */
  .stats-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 16px;
    transition: all 0.2s ease;
  }

  .stat-card:hover {
    border-color: hsl(var(--border-hover));
    transform: translateY(-2px);
    box-shadow: 0 8px 24px hsl(var(--background) / 0.4);
  }

  .progress-card {
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
  }

  .progress-ring {
    position: relative;
    width: 80px;
    height: 80px;
  }

  .progress-ring svg {
    transform: rotate(-90deg);
    width: 100%;
    height: 100%;
  }

  .progress-ring circle {
    fill: none;
    stroke-width: 8;
    stroke-linecap: round;
  }

  .progress-ring .bg {
    stroke: hsl(var(--muted));
  }

  .progress-ring .progress {
    stroke: hsl(var(--accent));
    stroke-dasharray: 251.2;
    stroke-dashoffset: calc(251.2 * (1 - var(--progress) / 100));
    transition: stroke-dashoffset 0.6s ease;
  }

  .progress-value {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 18px;
    font-weight: 700;
    color: hsl(var(--foreground));
  }

  .progress-label {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .label-main {
    font-size: 14px;
    font-weight: 600;
    color: hsl(var(--foreground));
  }

  .label-sub {
    font-size: 12px;
    color: hsl(var(--foreground-muted));
  }

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: 12px;
    flex-shrink: 0;
  }

  .stat-icon.total {
    background: hsl(var(--primary) / 0.15);
    color: hsl(var(--primary));
  }

  .stat-icon.applied {
    background: hsl(var(--success) / 0.15);
    color: hsl(var(--success));
  }

  .stat-icon.pending {
    background: hsl(var(--warning) / 0.15);
    color: hsl(var(--warning));
  }

  .stat-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-number {
    font-size: 24px;
    font-weight: 700;
    color: hsl(var(--foreground));
    line-height: 1;
  }

  .stat-label {
    font-size: 12px;
    color: hsl(var(--foreground-muted));
  }

  /* System Card */
  .system-card {
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 16px;
    padding: 20px;
  }

  .system-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
    color: hsl(var(--foreground));
  }

  .system-header h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }

  .system-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 16px;
  }

  .system-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: hsl(var(--surface));
    border-radius: 10px;
  }

  .system-item > :global(svg) {
    color: hsl(var(--foreground-muted));
    flex-shrink: 0;
  }

  .system-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .info-label {
    font-size: 11px;
    color: hsl(var(--foreground-muted));
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .info-value {
    font-size: 13px;
    font-weight: 500;
    color: hsl(var(--foreground));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .info-value.privilege.admin {
    color: hsl(var(--success));
  }

  /* Categories Section */
  .categories-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .section-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: hsl(var(--foreground));
    margin: 0;
  }

  .section-subtitle {
    font-size: 13px;
    color: hsl(var(--foreground-muted));
  }

  .categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .category-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 16px;
    cursor: pointer;
    transition: all 0.25s ease;
    text-align: left;
    overflow: hidden;
  }

  .category-card:hover {
    border-color: hsl(var(--accent) / 0.5);
    transform: translateY(-4px);
    box-shadow: 0 12px 32px hsl(var(--background) / 0.5);
  }

  .category-card:hover .card-arrow {
    opacity: 1;
    transform: translateX(0);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .category-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: hsl(var(--accent) / 0.15);
    color: hsl(var(--accent));
    border-radius: 12px;
  }

  .category-badge {
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 600;
    border-radius: 20px;
    background: hsl(var(--muted));
    color: hsl(var(--foreground-muted));
  }

  .category-badge.complete {
    background: hsl(var(--success) / 0.15);
    color: hsl(var(--success));
  }

  .card-content h3 {
    font-size: 16px;
    font-weight: 600;
    color: hsl(var(--foreground));
    margin: 0;
  }

  .card-content p {
    font-size: 13px;
    color: hsl(var(--foreground-muted));
    margin: 4px 0 0 0;
    line-height: 1.4;
  }

  .card-progress {
    margin-top: auto;
  }

  .progress-bar {
    height: 4px;
    background: hsl(var(--muted));
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, hsl(var(--accent)), hsl(var(--primary)));
    border-radius: 2px;
    transition: width 0.4s ease;
  }

  .card-arrow {
    position: absolute;
    right: 16px;
    bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: hsl(var(--accent));
    color: white;
    border-radius: 8px;
    opacity: 0;
    transform: translateX(-8px);
    transition: all 0.25s ease;
  }

  /* Footer */
  .overview-footer {
    margin-top: auto;
    padding-top: 16px;
  }

  .footer-tip {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 18px;
    background: hsl(var(--surface));
    border-radius: 12px;
    font-size: 13px;
    color: hsl(var(--foreground-muted));
  }

  .footer-tip > :global(svg) {
    color: hsl(var(--accent));
    flex-shrink: 0;
  }

  .footer-tip p {
    margin: 0;
    line-height: 1.5;
  }

  .footer-tip strong {
    color: hsl(var(--foreground));
  }

  /* Responsive */
  @media (max-width: 768px) {
    .overview {
      padding: 16px;
      gap: 16px;
    }

    .welcome-text h1 {
      font-size: 24px;
    }

    .stats-row {
      grid-template-columns: repeat(2, 1fr);
    }

    .progress-card {
      grid-column: span 2;
    }
  }
</style>
