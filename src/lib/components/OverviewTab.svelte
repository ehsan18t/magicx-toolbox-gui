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

<div class="mx-auto flex w-full max-w-[1400px] flex-col gap-6 p-6">
  <!-- Welcome Header -->
  <header class="flex flex-wrap items-start justify-between gap-4">
    <div>
      <h1 class="m-0 text-3xl font-bold tracking-tight text-foreground">Welcome back!</h1>
      <p class="mt-1 mb-0 text-sm text-foreground-muted">
        Manage your Windows tweaks and optimizations
      </p>
    </div>
    {#if $pendingRebootCount > 0}
      <div
        class="flex items-center gap-2 rounded-lg bg-warning/15 px-4 py-2 text-sm font-medium text-warning"
      >
        <Icon icon="mdi:restart-alert" width="20" />
        <span>{$pendingRebootCount} pending reboot</span>
      </div>
    {/if}
  </header>

  <!-- Stats Cards Row -->
  <section class="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-4">
    <!-- Progress Ring Card -->
    <div
      class="flex flex-col items-center gap-3 rounded-2xl border border-border bg-card p-5 text-center transition-all duration-200 hover:-translate-y-0.5 hover:border-border-hover hover:shadow-lg"
    >
      <div class="progress-ring relative h-20 w-20" style="--progress: {progressPercent}">
        <svg viewBox="0 0 100 100" class="h-full w-full -rotate-90">
          <circle class="fill-none stroke-[hsl(var(--muted))] stroke-8" cx="50" cy="50" r="40" />
          <circle
            class="progress-circle stroke-round fill-none stroke-accent stroke-8"
            cx="50"
            cy="50"
            r="40"
          />
        </svg>
        <span
          class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-lg font-bold text-foreground"
          >{progressPercent}%</span
        >
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-sm font-semibold text-foreground">Optimization</span>
        <span class="text-xs text-foreground-muted">Progress</span>
      </div>
    </div>

    <!-- Total Tweaks -->
    <div
      class="flex items-center gap-4 rounded-2xl border border-border bg-card p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-border-hover hover:shadow-lg"
    >
      <div
        class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-primary/15 text-primary"
      >
        <Icon icon="mdi:tune-vertical" width="24" />
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-2xl leading-none font-bold text-foreground">{$tweakStats.total}</span>
        <span class="text-xs text-foreground-muted">Total Tweaks</span>
      </div>
    </div>

    <!-- Applied Tweaks -->
    <div
      class="flex items-center gap-4 rounded-2xl border border-border bg-card p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-border-hover hover:shadow-lg"
    >
      <div
        class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-success/15 text-success"
      >
        <Icon icon="mdi:check-circle" width="24" />
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-2xl leading-none font-bold text-foreground">{$tweakStats.applied}</span>
        <span class="text-xs text-foreground-muted">Applied</span>
      </div>
    </div>

    <!-- Pending Tweaks -->
    <div
      class="flex items-center gap-4 rounded-2xl border border-border bg-card p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-border-hover hover:shadow-lg"
    >
      <div
        class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-warning/15 text-warning"
      >
        <Icon icon="mdi:clock-outline" width="24" />
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-2xl leading-none font-bold text-foreground">{$tweakStats.pending}</span>
        <span class="text-xs text-foreground-muted">Pending</span>
      </div>
    </div>
  </section>

  <!-- System Info Card -->
  <section class="rounded-2xl border border-border bg-card p-5">
    <div class="mb-4 flex items-center gap-2.5 text-foreground">
      <Icon icon="mdi:monitor" width="20" />
      <h2 class="m-0 text-base font-semibold">System Information</h2>
    </div>
    <div class="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-4">
      <div class="flex items-center gap-3 rounded-lg bg-surface p-3">
        <Icon icon="mdi:microsoft-windows" width="18" class="shrink-0 text-foreground-muted" />
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="text-xs tracking-wide text-foreground-muted uppercase">Operating System</span
          >
          <span class="truncate text-sm font-medium text-foreground"
            >{$systemStore?.windows?.product_name ?? "Windows"}</span
          >
        </div>
      </div>
      <div class="flex items-center gap-3 rounded-lg bg-surface p-3">
        <Icon icon="mdi:tag" width="18" class="shrink-0 text-foreground-muted" />
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="text-xs tracking-wide text-foreground-muted uppercase">Version</span>
          <span class="truncate text-sm font-medium text-foreground"
            >{$systemStore?.windows?.display_version ?? ""} (Build {$systemStore?.windows
              ?.build_number ?? ""})</span
          >
        </div>
      </div>
      <div class="flex items-center gap-3 rounded-lg bg-surface p-3">
        <Icon icon="mdi:account" width="18" class="shrink-0 text-foreground-muted" />
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="text-xs tracking-wide text-foreground-muted uppercase">User</span>
          <span class="truncate text-sm font-medium text-foreground"
            >{$systemStore?.username ?? ""}@{$systemStore?.computer_name ?? ""}</span
          >
        </div>
      </div>
      <div class="flex items-center gap-3 rounded-lg bg-surface p-3">
        <Icon icon="mdi:shield-check" width="18" class="shrink-0 text-foreground-muted" />
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="text-xs tracking-wide text-foreground-muted uppercase">Privileges</span>
          <span
            class="truncate text-sm font-medium {$systemStore?.is_admin
              ? 'text-success'
              : 'text-warning'}"
          >
            {$systemStore?.is_admin ? "Administrator" : "Standard User"}
          </span>
        </div>
      </div>
    </div>
  </section>

  <!-- Categories Grid -->
  <section class="flex flex-col gap-4">
    <div class="flex items-baseline gap-3">
      <h2 class="m-0 text-lg font-semibold text-foreground">Categories</h2>
      <span class="text-sm text-foreground-muted">{$categoriesStore.length} available</span>
    </div>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-4">
      {#each $categoriesStore as category (category.id)}
        {@const stats = $categoryStats[category.id]}
        {@const progress = stats?.total > 0 ? (stats.applied / stats.total) * 100 : 0}
        <button
          class="group relative flex cursor-pointer flex-col gap-3 overflow-hidden rounded-2xl border border-border bg-card p-5 text-left transition-all duration-250 hover:-translate-y-1 hover:border-accent/50 hover:shadow-xl"
          onclick={() => navigateToCategory(category.id)}
        >
          <div class="flex items-center justify-between">
            <div
              class="flex h-11 w-11 items-center justify-center rounded-xl bg-accent/15 text-accent"
            >
              <Icon icon={category.icon || "mdi:folder"} width="24" />
            </div>
            <span
              class="rounded-full px-2.5 py-1 text-xs font-semibold {progress === 100 &&
              stats?.total > 0
                ? 'bg-success/15 text-success'
                : 'bg-[hsl(var(--muted))] text-foreground-muted'}"
            >
              {stats?.applied ?? 0}/{stats?.total ?? 0}
            </span>
          </div>
          <div>
            <h3 class="m-0 text-base font-semibold text-foreground">{category.name}</h3>
            <p class="mt-1 mb-0 text-sm leading-relaxed text-foreground-muted">
              {category.description}
            </p>
          </div>
          <div class="mt-auto">
            <div class="h-1 overflow-hidden rounded-sm bg-[hsl(var(--muted))]">
              <div
                class="h-full rounded-sm bg-linear-to-r from-accent to-primary transition-[width] duration-400"
                style="width: {progress}%"
              ></div>
            </div>
          </div>
          <div
            class="absolute right-4 bottom-4 flex h-8 w-8 -translate-x-2 items-center justify-center rounded-lg bg-accent text-white opacity-0 transition-all duration-250 group-hover:translate-x-0 group-hover:opacity-100"
          >
            <Icon icon="mdi:arrow-right" width="18" />
          </div>
        </button>
      {/each}
    </div>
  </section>

  <!-- Footer Info -->
  <footer class="mt-auto pt-4">
    <div
      class="flex items-center gap-3 rounded-xl bg-surface px-4 py-3 text-sm text-foreground-muted"
    >
      <Icon icon="mdi:lightbulb-outline" width="18" class="shrink-0 text-accent" />
      <p class="m-0 leading-relaxed">
        <strong class="text-foreground">Tip:</strong> Changes are backed up automatically. Hover over
        the sidebar to expand it.
      </p>
    </div>
  </footer>
</div>

<style>
  .progress-circle {
    stroke-dasharray: 251.2;
    stroke-dashoffset: calc(251.2 * (1 - var(--progress) / 100));
    transition: stroke-dashoffset 0.6s ease;
    stroke-linecap: round;
  }
</style>
