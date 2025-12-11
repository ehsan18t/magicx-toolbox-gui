<script lang="ts">
  import Icon from "@iconify/svelte";

  const { total, applied, pending } = $props<{
    total: number;
    applied: number;
    pending: number;
  }>();

  const percentage = $derived(total > 0 ? Math.round((applied / total) * 100) : 0);
</script>

<div
  class="flex items-center gap-5 rounded-lg border border-border bg-card px-5 py-4 max-sm:flex-col max-sm:p-4"
>
  <!-- Main circular stat -->
  <div class="flex flex-col items-center gap-1">
    <div class="stat-circle relative h-15 w-15">
      <svg viewBox="0 0 36 36" class="h-full w-full">
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
      <span
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-sm font-bold text-foreground"
        >{percentage}%</span
      >
    </div>
    <div class="text-xs font-medium tracking-wide text-foreground-muted uppercase">Applied</div>
  </div>

  <!-- Divider -->
  <div class="h-12 w-px bg-border max-sm:h-px max-sm:w-full"></div>

  <!-- Stats group -->
  <div class="flex gap-6 max-sm:flex-wrap max-sm:justify-center">
    <div class="flex flex-col gap-0.5">
      <div class="flex items-center gap-1.5 text-lg font-semibold text-foreground-muted">
        <Icon icon="mdi:tune-vertical" width="16" />
        {total}
      </div>
      <div class="text-xs font-medium tracking-wide text-foreground-muted uppercase">
        Total Tweaks
      </div>
    </div>

    <div class="flex flex-col gap-0.5">
      <div class="flex items-center gap-1.5 text-lg font-semibold text-success">
        <Icon icon="mdi:check-circle" width="16" />
        {applied}
      </div>
      <div class="text-xs font-medium tracking-wide text-foreground-muted uppercase">Applied</div>
    </div>

    <div class="flex flex-col gap-0.5">
      <div class="flex items-center gap-1.5 text-lg font-semibold text-foreground-muted">
        <Icon icon="mdi:circle-outline" width="16" />
        {pending}
      </div>
      <div class="text-xs font-medium tracking-wide text-foreground-muted uppercase">Pending</div>
    </div>
  </div>
</div>

<style>
  .stat-circle {
    width: 60px;
    height: 60px;
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
</style>
