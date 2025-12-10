<script lang="ts">
  import Icon from "@iconify/svelte";

  const { total, applied, pending } = $props<{
    total: number;
    applied: number;
    pending: number;
  }>();

  const percentage = $derived(total > 0 ? Math.round((applied / total) * 100) : 0);
</script>

<div class="stats-card">
  <div class="stat-item main">
    <div class="stat-circle">
      <svg viewBox="0 0 36 36" class="circular-chart">
        <path
          class="circle-bg"
          d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
        />
        <path
          class="circle"
          stroke-dasharray="{percentage}, 100"
          d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
        />
      </svg>
      <span class="percentage">{percentage}%</span>
    </div>
    <div class="stat-label">Applied</div>
  </div>

  <div class="stat-divider"></div>

  <div class="stat-group">
    <div class="stat-item">
      <div class="stat-value total">
        <Icon icon="mdi:tune-vertical" width="16" />
        {total}
      </div>
      <div class="stat-label">Total Tweaks</div>
    </div>

    <div class="stat-item">
      <div class="stat-value applied">
        <Icon icon="mdi:check-circle" width="16" />
        {applied}
      </div>
      <div class="stat-label">Applied</div>
    </div>

    <div class="stat-item">
      <div class="stat-value pending">
        <Icon icon="mdi:circle-outline" width="16" />
        {pending}
      </div>
      <div class="stat-label">Pending</div>
    </div>
  </div>
</div>

<style>
  .stats-card {
    display: flex;
    align-items: center;
    gap: 20px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    padding: 16px 20px;
  }

  .stat-item.main {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .stat-circle {
    position: relative;
    width: 60px;
    height: 60px;
  }

  .circular-chart {
    width: 100%;
    height: 100%;
  }

  .circle-bg {
    fill: none;
    stroke: hsl(var(--muted));
    stroke-width: 3;
  }

  .circle {
    fill: none;
    stroke: hsl(var(--primary));
    stroke-width: 3;
    stroke-linecap: round;
    transform: rotate(-90deg);
    transform-origin: center;
    transition: stroke-dasharray 0.5s ease;
  }

  .percentage {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 14px;
    font-weight: 700;
    color: hsl(var(--foreground));
  }

  .stat-divider {
    width: 1px;
    height: 50px;
    background: hsl(var(--border));
  }

  .stat-group {
    display: flex;
    gap: 24px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-value {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 18px;
    font-weight: 600;
    color: hsl(var(--foreground));
  }

  .stat-value.total {
    color: hsl(var(--muted-foreground));
  }

  .stat-value.applied {
    color: hsl(142 76% 36%);
  }

  .stat-value.pending {
    color: hsl(var(--muted-foreground));
  }

  .stat-label {
    font-size: 11px;
    font-weight: 500;
    color: hsl(var(--muted-foreground));
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  @media (max-width: 640px) {
    .stats-card {
      flex-direction: column;
      padding: 16px;
    }

    .stat-divider {
      width: 100%;
      height: 1px;
    }

    .stat-group {
      flex-wrap: wrap;
      justify-content: center;
    }
  }
</style>
