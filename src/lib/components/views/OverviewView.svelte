<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { categoriesStore, getCategoryStats, loadingStateStore, systemStore } from "$lib/stores/tweaks.svelte";
  import ScrollingText from "./ScrollingText.svelte";
  import SystemInfoCard from "./SystemInfoCard.svelte";

  // Get category stats reactively
  const categoryStats = $derived(getCategoryStats());

  // Loading states
  const systemInfoLoading = $derived(loadingStateStore.systemInfoLoading);
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

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
      {#if systemInfoLoading}
        <!-- Skeleton loading for system info -->
        <div
          class="grid grid-cols-2 gap-4 gap-y-6 md:grid-cols-[2fr_1fr_1fr_1fr] md:gap-0 md:divide-x md:divide-border/50"
        >
          {#each [0, 1, 2, 3] as i (`system-skeleton-${i}`)}
            <div class="flex flex-col gap-1.5 {i === 0 ? 'md:pr-4' : i === 3 ? 'md:pl-4' : 'md:px-4'}">
              <span class="flex items-center gap-2">
                <div class="animate-pulse h-3.5 w-3.5 rounded bg-surface/80"></div>
                <div class="animate-pulse h-3 w-24 rounded bg-surface/80"></div>
              </span>
              <div class="flex flex-col gap-1">
                <div class="animate-pulse h-5 w-32 rounded bg-surface/60"></div>
                <div class="animate-pulse h-3 w-20 rounded bg-surface/40"></div>
              </div>
            </div>
          {/each}
        </div>
      {:else}
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
                {systemStore.info?.windows?.product_name?.replace("Windows ", "Win ") ?? "Windows"}
              </span>
              <span class="text-xs text-foreground-muted">
                {systemStore.info?.windows?.display_version ?? ""} ({systemStore.info?.windows?.build_number ?? ""})
              </span>
            </div>
          </div>

          <!-- Device -->
          <div class="flex flex-col gap-1.5 md:px-4">
            <span class="flex items-center gap-2 text-[10px] font-bold tracking-wider text-foreground-muted uppercase">
              <Icon
                icon={systemStore.info?.device?.pc_type === "Laptop" ? "mdi:laptop" : "mdi:desktop-tower-monitor"}
                width="14"
                class="text-accent"
              />
              Device
            </span>
            <div class="flex flex-col">
              <span class="truncate text-sm font-semibold text-foreground">
                {systemStore.info?.device?.model ?? systemStore.info?.computer_name ?? "Unknown"}
              </span>
              <span class="truncate text-xs text-foreground-muted">
                {systemStore.info?.device?.manufacturer ?? "Unknown"}
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
                {formatUptime(systemStore.info?.windows?.uptime_seconds ?? 0)}
              </span>
              <span class="text-xs text-foreground-muted">Since boot</span>
            </div>
          </div>

          <!-- User -->
          <div class="flex flex-col gap-1.5 md:pl-4">
            <span class="flex items-center gap-2 text-[10px] font-bold tracking-wider text-foreground-muted uppercase">
              <Icon
                icon={systemStore.info?.is_admin ? "mdi:shield-check" : "mdi:account"}
                width="14"
                class={systemStore.info?.is_admin ? "text-success" : "text-warning"}
              />
              User
            </span>
            <div class="flex flex-col">
              <span class="truncate text-sm font-semibold text-foreground">
                {systemStore.info?.username ?? "User"}
              </span>
              <span class="text-xs {systemStore.info?.is_admin ? 'text-success' : 'text-warning'}">
                {systemStore.info?.is_admin ? "Admin" : "Standard"}
              </span>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Hardware Section -->
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h2 class="m-0 text-lg font-semibold text-foreground">Hardware</h2>
    </div>

    {#if systemInfoLoading}
      <!-- Skeleton loading for hardware -->
      <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
        {#each [0, 1, 2, 3, 4, 5] as i (`hardware-skeleton-${i}`)}
          <div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
            <div class="animate-pulse h-10 w-10 rounded-lg bg-surface/60"></div>
            <div class="flex min-w-0 flex-1 flex-col gap-1.5">
              <div class="animate-pulse h-3 w-16 rounded bg-surface/80"></div>
              <div class="animate-pulse h-4 w-40 rounded bg-surface/60"></div>
              <div class="flex items-center gap-2">
                <div class="animate-pulse h-3 w-16 rounded bg-surface/40"></div>
                <div class="h-1 w-1 rounded-full bg-border"></div>
                <div class="animate-pulse h-3 w-20 rounded bg-surface/40"></div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
        <!-- CPU -->
        <SystemInfoCard
          icon="mdi:cpu-64-bit"
          label="Processor"
          title={systemStore.info?.hardware?.cpu?.name ?? "Unknown CPU"}
        >
          <span>{systemStore.info?.hardware?.cpu?.cores ?? 0} Cores</span>
          <span class="h-1 w-1 rounded-full bg-border"></span>
          <span>{formatClockSpeed(systemStore.info?.hardware?.cpu?.max_clock_mhz ?? 0)}</span>
        </SystemInfoCard>

        <!-- GPU(s) -->
        {#if systemStore.info?.hardware?.gpu && systemStore.info.hardware.gpu.length > 0}
          {#each systemStore.info.hardware.gpu as gpu, i (gpu.name)}
            <SystemInfoCard
              icon="mdi:expansion-card"
              label="Graphics{systemStore.info.hardware.gpu.length > 1 ? ` ${i + 1}` : ''}"
              title={gpu.name}
            >
              <ScrollingText>
                <span
                  >{#if gpu.memory_gb > 0}{gpu.memory_gb} GB{:else}Shared{/if}</span
                >

                {#if i === 0 && systemStore.info?.hardware?.monitors && systemStore.info.hardware.monitors.length > 0}
                  {#each systemStore.info.hardware.monitors as monitor, monitorIndex (monitor.name + monitorIndex)}
                    <span class="h-1 w-1 rounded-full bg-border"></span>
                    <span use:tooltip={`${monitor.name} - ${monitor.resolution}`}>
                      {monitor.name} <span class="text-muted-foreground ml-1">{monitor.resolution}</span>
                    </span>
                    {#if monitor.refresh_rate > 0}
                      <span class="text-muted-foreground bg-muted self-center rounded-md px-1.5 py-0.5 text-xs">
                        {monitor.refresh_rate}Hz
                      </span>
                    {/if}
                  {/each}
                {:else if gpu.refresh_rate > 0}
                  <span class="h-1 w-1 rounded-full bg-border"></span>
                  <span>{gpu.refresh_rate}Hz</span>
                {/if}
              </ScrollingText>
            </SystemInfoCard>
          {/each}
        {/if}

        <!-- Motherboard -->
        <SystemInfoCard
          icon="bi:motherboard"
          label="Motherboard"
          title={systemStore.info?.hardware?.motherboard?.product ?? "Unknown"}
        >
          <span class="truncate">{systemStore.info?.hardware?.motherboard?.manufacturer ?? "Unknown"}</span>
          {#if systemStore.info?.hardware?.motherboard?.bios_version}
            <span class="h-1 w-1 rounded-full bg-border"></span>
            <span class="truncate">BIOS: {systemStore.info?.hardware?.motherboard?.bios_version}</span>
          {/if}
        </SystemInfoCard>

        <!-- Memory -->
        <SystemInfoCard
          icon="ri:ram-line"
          label="Memory"
          title="{systemStore.info?.hardware?.memory?.total_gb ?? 0} GB {systemStore.info?.hardware?.memory
            ?.memory_type ?? ''}"
        >
          <span>{systemStore.info?.hardware?.memory?.speed_mhz ?? 0} MHz</span>
          <span class="h-1 w-1 rounded-full bg-border"></span>
          <span>{systemStore.info?.hardware?.memory?.slots_used ?? 0} / 4 Slots</span>
        </SystemInfoCard>

        <!-- Storage Drives -->
        {#if systemStore.info?.hardware?.disks && systemStore.info.hardware.disks.length > 0}
          {#each systemStore.info.hardware.disks as disk, i (disk.model)}
            <SystemInfoCard
              icon={disk.drive_type === "SSD" ? "mdi:harddisk" : "mdi:harddisk-plus"}
              label="Storage{systemStore.info.hardware.disks.length > 1 ? ` ${i + 1}` : ''}"
              title={disk.model}
            >
              {#snippet headerExtra()}
                {#if disk.health_status}
                  <span
                    class="text-xs font-medium {disk.health_status === 'Healthy' ? 'text-success' : 'text-warning'}"
                  >
                    {disk.health_status}
                  </span>
                {/if}
              {/snippet}

              <span>{formatStorage(disk.size_gb)}</span>
              {#if disk.interface_type && disk.interface_type !== "Unknown"}
                <span class="h-1 w-1 rounded-full bg-border"></span>
                <span>{disk.interface_type}</span>
              {/if}
            </SystemInfoCard>
          {/each}
        {/if}

        <!-- Network -->
        {#if systemStore.info?.hardware?.network && systemStore.info.hardware.network.length > 0}
          {#each systemStore.info.hardware.network as net, i (net.mac_address)}
            <SystemInfoCard
              icon="mdi:ethernet"
              label="Network{systemStore.info.hardware.network.length > 1 ? ` ${i + 1}` : ''}"
              title={net.name}
            >
              <span>{net.ip_address}</span>
              <span class="h-1 w-1 rounded-full bg-border"></span>
              <span class="truncate font-mono text-[10px] uppercase">{net.mac_address}</span>
            </SystemInfoCard>
          {/each}
        {/if}
      </div>
    {/if}
  </div>

  <!-- Categories Section -->
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h2 class="m-0 text-lg font-semibold text-foreground">Tweak Categories</h2>
      {#if !tweaksLoading}
        <span class="text-xs text-foreground-muted">{categoriesStore.list.length} available</span>
      {/if}
    </div>
    {#if tweaksLoading}
      <!-- Skeleton loading for categories -->
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {#each [0, 1, 2, 3, 4, 5] as i (`category-skeleton-${i}`)}
          <div class="flex items-start gap-3 rounded-xl border border-border bg-card p-4">
            <div class="animate-pulse h-10 w-10 rounded-lg bg-surface/60"></div>
            <div class="flex min-w-0 flex-1 flex-col gap-2">
              <div class="flex items-center justify-between">
                <div class="animate-pulse h-4 w-24 rounded bg-surface/60"></div>
                <div class="animate-pulse h-3 w-8 rounded bg-surface/40"></div>
              </div>
              <div class="flex flex-col gap-1">
                <div class="animate-pulse h-3 w-full rounded bg-surface/40"></div>
                <div class="animate-pulse h-3 w-3/4 rounded bg-surface/40"></div>
              </div>
              <div class="animate-pulse h-1 w-full rounded-full bg-surface/30"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {#each categoriesStore.list as category (category.id)}
          {@const stats = categoryStats[category.id]}
          {@const progress = stats?.total > 0 ? (stats.applied / stats.total) * 100 : 0}
          <button
            class="group relative flex cursor-pointer items-start gap-3 overflow-hidden rounded-xl border border-border bg-card p-4 text-left transition-all duration-200 hover:border-accent/50 hover:shadow-md"
            onclick={() => navigationStore.navigateToCategory(category.id)}
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
              <div class="bg-muted h-1 overflow-hidden rounded-full">
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
    {/if}
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
