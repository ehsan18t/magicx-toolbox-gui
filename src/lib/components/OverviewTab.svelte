<script lang="ts">
  import { navigateToCategory } from "$lib/stores/navigation";
  import { categoriesStore, categoryStats, systemStore } from "$lib/stores/tweaks";
  import Icon from "./Icon.svelte";

  // Format clock speed
  const formatClockSpeed = (mhz: number) => {
    if (mhz >= 1000) {
      return `${(mhz / 1000).toFixed(1)} GHz`;
    }
    return `${mhz} MHz`;
  };

  // Format storage size
  const formatStorage = (gb: number) => {
    if (gb >= 1000) {
      return `${(gb / 1000).toFixed(1)} TB`;
    }
    return `${gb.toFixed(0)} GB`;
  };

  // Format uptime
  const formatUptime = (seconds: number): string => {
    if (!seconds || seconds <= 0) return "Unknown";
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };
</script>

<div class="flex h-full w-full flex-col gap-5 overflow-y-auto p-5">
  <!-- System Overview -->
  <div class="flex flex-col gap-2">
    <h2 class="m-0 text-lg font-semibold text-foreground">System</h2>
    <div class="rounded-xl border border-border bg-card p-4">
      <div
        class="grid grid-cols-2 gap-4 gap-y-6 md:grid-cols-[2fr_1fr_1fr_1fr] md:gap-0 md:divide-x md:divide-border/50"
      >
        <!-- OS -->
        <div class="flex flex-col gap-1.5 md:pr-4">
          <span class="flex items-center gap-2 text-[10px] font-bold tracking-wider text-foreground-muted uppercase">
            <Icon icon="mdi:microsoft-windows" width="14" class="text-accent" />
            Operating System
          </span>
          <div class="flex flex-col">
            <span class="text-sm font-semibold text-foreground">
              {$systemStore?.windows?.product_name?.replace("Windows ", "Win ") ?? "Windows"}
            </span>
            <span class="text-xs text-foreground-muted">
              {$systemStore?.windows?.display_version ?? ""} ({$systemStore?.windows?.build_number ?? ""})
            </span>
          </div>
        </div>

        <!-- Device -->
        <div class="flex flex-col gap-1.5 md:px-4">
          <span class="flex items-center gap-2 text-[10px] font-bold tracking-wider text-foreground-muted uppercase">
            <Icon
              icon={$systemStore?.device?.pc_type === "Laptop" ? "mdi:laptop" : "mdi:desktop-tower-monitor"}
              width="14"
              class="text-accent"
            />
            Device
          </span>
          <div class="flex flex-col">
            <span class="truncate text-sm font-semibold text-foreground">
              {$systemStore?.device?.model ?? $systemStore?.computer_name ?? "Unknown"}
            </span>
            <span class="truncate text-xs text-foreground-muted">
              {$systemStore?.device?.manufacturer ?? "Unknown"}
            </span>
          </div>
        </div>

        <!-- Uptime -->
        <div class="flex flex-col gap-1.5 md:px-4">
          <span class="flex items-center gap-2 text-[10px] font-bold tracking-wider text-foreground-muted uppercase">
            <Icon icon="mdi:timer-outline" width="14" class="text-success" />
            Uptime
          </span>
          <div class="flex flex-col">
            <span class="text-sm font-semibold text-foreground">
              {formatUptime($systemStore?.windows?.uptime_seconds ?? 0)}
            </span>
            <span class="text-xs text-foreground-muted">Since boot</span>
          </div>
        </div>

        <!-- User -->
        <div class="flex flex-col gap-1.5 md:pl-4">
          <span class="flex items-center gap-2 text-[10px] font-bold tracking-wider text-foreground-muted uppercase">
            <Icon
              icon={$systemStore?.is_admin ? "mdi:shield-check" : "mdi:account"}
              width="14"
              class={$systemStore?.is_admin ? "text-success" : "text-warning"}
            />
            User
          </span>
          <div class="flex flex-col">
            <span class="truncate text-sm font-semibold text-foreground">
              {$systemStore?.username ?? "User"}
            </span>
            <span class="text-xs {$systemStore?.is_admin ? 'text-success' : 'text-warning'}">
              {$systemStore?.is_admin ? "Admin" : "Standard"}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Hardware Section -->
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h2 class="m-0 text-lg font-semibold text-foreground">Hardware</h2>
    </div>

    <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
      <!-- CPU -->
      <div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
          <Icon icon="mdi:cpu-64-bit" width="20" class="text-accent" />
        </div>
        <div class="flex min-w-0 flex-1 flex-col">
          <span class="text-xs font-medium text-foreground-muted uppercase">Processor</span>
          <div class="flex items-center justify-between gap-2">
            <h3 class="line-clamp-1 text-sm font-semibold text-foreground" title={$systemStore?.hardware?.cpu?.name}>
              {$systemStore?.hardware?.cpu?.name ?? "Unknown CPU"}
            </h3>
          </div>
          <div class="mt-0.5 flex items-center gap-2 text-xs text-foreground-muted">
            <span>{$systemStore?.hardware?.cpu?.cores ?? 0} Cores</span>
            <span class="h-1 w-1 rounded-full bg-border"></span>
            <span>{formatClockSpeed($systemStore?.hardware?.cpu?.max_clock_mhz ?? 0)}</span>
          </div>
        </div>
      </div>

      <!-- GPU(s) -->
      {#if $systemStore?.hardware?.gpu && $systemStore.hardware.gpu.length > 0}
        {#each $systemStore.hardware.gpu as gpu, i (i)}
          <div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
              <Icon icon="mdi:expansion-card" width="20" class="text-accent" />
            </div>
            <div class="flex min-w-0 flex-1 flex-col">
              <span class="text-xs font-medium text-foreground-muted uppercase">
                Graphics{$systemStore.hardware.gpu.length > 1 ? ` ${i + 1}` : ""}
              </span>
              <h3 class="line-clamp-1 text-sm font-semibold text-foreground" title={gpu.name}>
                {gpu.name}
              </h3>
              <div class="mt-0.5 flex items-center gap-2 text-xs text-foreground-muted">
                <span
                  >{#if gpu.memory_gb > 0}{gpu.memory_gb} GB{:else}Shared{/if}</span
                >
                {#if gpu.refresh_rate > 0}
                  <span class="h-1 w-1 rounded-full bg-border"></span>
                  <span>{gpu.refresh_rate}Hz</span>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      {/if}

      <!-- Memory -->
      <div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
          <Icon icon="ri:ram-line" width="20" class="text-accent" />
        </div>
        <div class="flex min-w-0 flex-1 flex-col">
          <span class="text-xs font-medium text-foreground-muted uppercase">Memory</span>
          <h3 class="truncate text-sm font-semibold text-foreground">
            {$systemStore?.hardware?.memory?.total_gb ?? 0} GB {$systemStore?.hardware?.memory?.memory_type ?? ""}
          </h3>
          <div class="mt-0.5 flex items-center gap-2 text-xs text-foreground-muted">
            <span>{$systemStore?.hardware?.memory?.speed_mhz ?? 0} MHz</span>
            <span class="h-1 w-1 rounded-full bg-border"></span>
            <span>{$systemStore?.hardware?.memory?.slots_used ?? 0} / 4 Slots</span>
          </div>
        </div>
      </div>

      <!-- Storage Drives -->
      {#if $systemStore?.hardware?.disks && $systemStore.hardware.disks.length > 0}
        {#each $systemStore.hardware.disks as disk, i (i)}
          <div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
              <Icon
                icon={disk.drive_type === "SSD" ? "mdi:harddisk" : "mdi:harddisk-plus"}
                width="20"
                class="text-accent"
              />
            </div>
            <div class="flex min-w-0 flex-1 flex-col">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-foreground-muted uppercase">
                  Storage{$systemStore.hardware.disks.length > 1 ? ` ${i + 1}` : ""}
                </span>
                {#if disk.health_status}
                  <span
                    class="text-[10px] font-medium {disk.health_status === 'Healthy' ? 'text-success' : 'text-warning'}"
                  >
                    {disk.health_status}
                  </span>
                {/if}
              </div>
              <h3 class="truncate text-sm font-semibold text-foreground" title={disk.model}>
                {disk.model}
              </h3>
              <div class="mt-0.5 flex items-center gap-2 text-xs text-foreground-muted">
                <span>{formatStorage(disk.size_gb)}</span>
                {#if disk.interface_type && disk.interface_type !== "Unknown"}
                  <span class="h-1 w-1 rounded-full bg-border"></span>
                  <span>{disk.interface_type}</span>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      {/if}

      <!-- Motherboard -->
      <div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
          <Icon icon="bi:motherboard" width="20" class="text-accent" />
        </div>
        <div class="flex min-w-0 flex-1 flex-col">
          <span class="text-xs font-medium text-foreground-muted uppercase">Motherboard</span>
          <h3 class="truncate text-sm font-semibold text-foreground">
            {$systemStore?.hardware?.motherboard?.product ?? "Unknown"}
          </h3>
          <div class="mt-0.5 truncate text-xs text-foreground-muted">
            {$systemStore?.hardware?.motherboard?.manufacturer ?? "Unknown"}
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Categories Section -->
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h2 class="m-0 text-lg font-semibold text-foreground">Tweak Categories</h2>
      <span class="text-xs text-foreground-muted">{$categoriesStore.length} available</span>
    </div>
    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {#each $categoriesStore as category (category.id)}
        {@const stats = $categoryStats[category.id]}
        {@const progress = stats?.total > 0 ? (stats.applied / stats.total) * 100 : 0}
        <button
          class="group relative flex cursor-pointer items-start gap-3 overflow-hidden rounded-xl border border-border bg-card p-4 text-left transition-all duration-200 hover:border-accent/50 hover:shadow-md"
          onclick={() => navigateToCategory(category.id)}
        >
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/15 text-accent">
            <Icon icon={category.icon || "mdi:folder"} width="20" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="mb-1 flex items-center justify-between gap-2">
              <h3 class="m-0 text-sm font-semibold text-foreground">{category.name}</h3>
              <span
                class="shrink-0 text-xs font-medium {progress === 100 && stats?.total > 0
                  ? 'text-success'
                  : 'text-foreground-muted'}"
              >
                {stats?.applied ?? 0}/{stats?.total ?? 0}
              </span>
            </div>
            <p class="m-0 mb-2 line-clamp-2 text-xs leading-relaxed text-foreground-muted">
              {category.description}
            </p>
            <div class="h-1 overflow-hidden rounded-full bg-[hsl(var(--muted))]">
              <div
                class="h-full rounded-full bg-accent transition-[width] duration-300"
                style="width: {progress}%"
              ></div>
            </div>
          </div>
          <Icon
            icon="mdi:chevron-right"
            width="18"
            class="shrink-0 text-foreground-muted opacity-0 transition-opacity group-hover:opacity-100"
          />
        </button>
      {/each}
    </div>
  </div>

  <!-- Footer Tip -->
  <div class="mt-auto flex items-center gap-2.5 rounded-lg bg-surface/50 px-3 py-2.5 text-xs text-foreground-muted">
    <Icon icon="mdi:lightbulb-outline" width="14" class="shrink-0 text-accent" />
    <span>
      <strong class="text-foreground">Tip:</strong> Changes are backed up automatically. Hover the sidebar to expand.
    </span>
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
