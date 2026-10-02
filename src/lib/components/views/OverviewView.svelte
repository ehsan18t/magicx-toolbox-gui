<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import {
    categoriesStore,
    getCategoryStats,
    loadingStateStore,
    pendingChangesStore,
    systemStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";

  const info = $derived(systemStore.info);
  const hw = $derived(info?.hardware);
  const categoryStats = $derived(getCategoryStats());
  const systemLoading = $derived(loadingStateStore.systemInfoLoading);

  const applied = $derived(tweaksStore.list.filter((t) => t.status.is_applied).length);
  const attention = $derived(tweaksStore.list.filter((t) => t.status.attention).length);
  const snapshots = $derived(tweaksStore.list.filter((t) => t.status.has_backup).length);

  const formatClock = (mhz: number) => (mhz >= 1000 ? `${(mhz / 1000).toFixed(1)} GHz` : `${mhz} MHz`);
  const formatStorage = (gb: number) => (gb >= 1000 ? `${(gb / 1000).toFixed(1)} TB` : `${gb.toFixed(0)} GB`);

  function formatUptime(seconds: number): string {
    if (!seconds || seconds <= 0) return "Unknown";
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  }

  interface HardwareRow {
    icon: string;
    label: string;
    value: string;
    detail: string;
    status?: { text: string; tone: string };
  }

  const hardware = $derived.by((): HardwareRow[] => {
    if (!hw) return [];
    const rows: HardwareRow[] = [
      {
        icon: "mdi:cpu-64-bit",
        label: "Processor",
        value: hw.cpu.name,
        detail: `${hw.cpu.cores} cores, ${hw.cpu.threads} threads · up to ${formatClock(hw.cpu.max_clock_mhz)}`,
      },
    ];
    hw.gpu.forEach((gpu, i) =>
      rows.push({
        icon: "mdi:expansion-card",
        label: hw.gpu.length > 1 ? `Graphics ${i + 1}` : "Graphics",
        value: gpu.name,
        detail: [
          gpu.memory_gb > 0 ? `${gpu.memory_gb} GB` : "Shared memory",
          gpu.driver_version && `Driver ${gpu.driver_version}`,
        ]
          .filter(Boolean)
          .join(" · "),
      }),
    );
    hw.monitors.forEach((m, i) =>
      rows.push({
        icon: "mdi:monitor",
        label: hw.monitors.length > 1 ? `Display ${i + 1}` : "Display",
        value: m.name,
        detail: [m.resolution, m.refresh_rate > 0 && `${m.refresh_rate} Hz`].filter(Boolean).join(" · "),
      }),
    );
    rows.push({
      icon: "ri:ram-line",
      label: "Memory",
      value: `${hw.memory.total_gb} GB ${hw.memory.memory_type}`.trim(),
      detail: [
        hw.memory.speed_mhz > 0 && `${hw.memory.speed_mhz} MHz`,
        hw.memory.slots_used > 0 && `${hw.memory.slots_used} modules`,
      ]
        .filter(Boolean)
        .join(" · "),
    });
    rows.push({
      icon: "bi:motherboard",
      label: "Motherboard",
      value: hw.motherboard.product,
      detail: [hw.motherboard.manufacturer, hw.motherboard.bios_version && `BIOS ${hw.motherboard.bios_version}`]
        .filter(Boolean)
        .join(" · "),
    });
    hw.disks.forEach((d, i) =>
      rows.push({
        icon: d.drive_type === "SSD" ? "mdi:harddisk" : "mdi:harddisk-plus",
        label: hw.disks.length > 1 ? `Storage ${i + 1}` : "Storage",
        value: d.model,
        detail: [formatStorage(d.size_gb), d.drive_type, d.interface_type !== "Unknown" && d.interface_type]
          .filter(Boolean)
          .join(" · "),
        status: d.health_status
          ? { text: d.health_status, tone: d.health_status === "Healthy" ? "text-success" : "text-warning" }
          : undefined,
      }),
    );
    hw.network.forEach((n, i) =>
      rows.push({
        icon: "mdi:ethernet",
        label: hw.network.length > 1 ? `Network ${i + 1}` : "Network",
        value: n.name,
        detail: [n.ip_address || "Not connected", n.mac_address].filter(Boolean).join(" · "),
      }),
    );
    return rows;
  });

  const tiles = $derived([
    {
      label: "Applied",
      value: `${applied}`,
      sub: `of ${tweaksStore.list.length} tweaks`,
      icon: "mdi:check-circle",
      tone: "text-accent",
      onclick: null,
    },
    {
      label: "Needs attention",
      value: `${attention}`,
      sub: attention ? "Open a category to resolve" : "All verified",
      icon: "mdi:alert-circle",
      tone: attention ? "text-error" : "text-success",
      onclick: null,
    },
    {
      label: "Snapshots",
      value: `${snapshots}`,
      sub: "restorable",
      icon: "mdi:history",
      tone: "text-foreground-muted",
      onclick: () => navigationStore.navigateToSnapshots(),
    },
    {
      label: "Ready to apply",
      value: `${pendingChangesStore.count}`,
      sub: pendingChangesStore.count ? "staged changes" : "nothing staged",
      icon: "mdi:arrow-right",
      tone: pendingChangesStore.count ? "text-warning" : "text-foreground-muted",
      onclick: null,
    },
  ]);
</script>

{#snippet skeleton(lines: number)}
  {#each Array.from({ length: lines }, (_, i) => i) as i (i)}
    <div class="h-4 animate-pulse rounded bg-muted" style="width: {90 - i * 15}%"></div>
  {/each}
{/snippet}

<PageLayout title="Overview" description={info ? `${info.computer_name} · ${info.username}` : "Your PC at a glance"}>
  <section class="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-2" aria-label="System">
    {#if systemLoading || !info}
      {#each [0, 1, 2, 3] as i (i)}
        <div class="space-y-2 rounded-lg border border-border bg-card p-3">{@render skeleton(2)}</div>
      {/each}
    {:else}
      {@const facts = [
        {
          icon: "mdi:microsoft-windows",
          label: "Windows",
          value: info.windows.product_name,
          detail: `${info.windows.display_version} · build ${info.windows.build_number}`,
        },
        {
          icon: info.device.pc_type === "Laptop" ? "mdi:laptop" : "mdi:desktop-tower-monitor",
          label: "Device",
          value: info.device.model || info.computer_name,
          detail: info.device.manufacturer,
        },
        {
          icon: "mdi:timer-outline",
          label: "Uptime",
          value: formatUptime(info.windows.uptime_seconds),
          detail: "since last restart",
        },
        {
          icon: info.is_admin ? "mdi:shield-check" : "mdi:account",
          label: "Account",
          value: info.username,
          detail: info.is_admin ? "Administrator" : "Standard user",
        },
      ]}
      {#each facts as f (f.label)}
        <div class="min-w-0 rounded-lg border border-border bg-card p-3">
          <div class="flex items-center gap-1.5 text-xs text-foreground-muted">
            <Icon icon={f.icon} width="14" class="shrink-0" />
            {f.label}
          </div>
          <div class="mt-1 text-sm font-semibold wrap-break-word">{f.value}</div>
          <div class="text-xs wrap-break-word text-foreground-muted">{f.detail}</div>
        </div>
      {/each}
    {/if}
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="overview-tweaks">
    <h2 id="overview-tweaks" class="m-0 text-base font-semibold">Your tweaks</h2>
    <div class="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-2">
      {#each tiles as t (t.label)}
        <svelte:element
          this={t.onclick ? "button" : "div"}
          type={t.onclick ? "button" : undefined}
          role={t.onclick ? undefined : "group"}
          class="flex min-w-0 flex-col items-start rounded-lg border border-border bg-card p-3 text-left {t.onclick
            ? 'cursor-pointer hover:border-border-hover'
            : ''}"
          onclick={t.onclick ?? undefined}
        >
          <span class="flex items-center gap-1.5 text-xs text-foreground-muted">
            <Icon icon={t.icon} width="14" class="shrink-0 {t.tone}" />
            {t.label}
          </span>
          <span class="mt-1 font-display text-2xl leading-none font-semibold tabular-nums">{t.value}</span>
          <span class="mt-1 text-xs text-foreground-muted">{t.sub}</span>
        </svelte:element>
      {/each}
    </div>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="overview-categories">
    <h2 id="overview-categories" class="m-0 text-base font-semibold">Categories</h2>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-2">
      {#each categoriesStore.list as category (category.id)}
        {@const s = categoryStats[category.id]}
        {@const progress = s?.total ? (s.applied / s.total) * 100 : 0}
        <button
          type="button"
          class="group flex min-w-0 cursor-pointer flex-col gap-2 rounded-lg border border-border bg-card p-3 text-left hover:border-border-hover"
          onclick={() => navigationStore.navigateToCategory(category.id)}
          aria-label="{category.name}: {s?.applied ?? 0} of {s?.total ?? 0} applied"
        >
          <span class="flex w-full items-center gap-2.5">
            <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent/12 text-accent">
              <Icon icon={category.icon || "mdi:folder"} width="18" />
            </span>
            <span class="min-w-0 flex-1 text-sm font-semibold wrap-break-word">{category.name}</span>
            <span
              class="shrink-0 text-xs tabular-nums {s && s.total > 0 && s.applied === s.total
                ? 'text-success'
                : 'text-foreground-muted'}"
            >
              {s?.applied ?? 0}/{s?.total ?? 0}
            </span>
          </span>
          <span class="line-clamp-2 text-xs text-foreground-muted">{category.description}</span>
          <span class="mt-auto block h-1 w-full overflow-hidden rounded-full bg-muted">
            <span class="block h-full rounded-full bg-accent transition-[width] duration-300" style="width: {progress}%"
            ></span>
          </span>
        </button>
      {/each}
    </div>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="overview-hardware">
    <div class="flex items-center justify-between gap-3">
      <h2 id="overview-hardware" class="m-0 text-base font-semibold">Hardware</h2>
      <div class="flex items-center gap-1">
        {#if systemStore.cachedAt && !systemLoading}
          <span class="text-xs text-foreground-subtle">
            Updated {new Date(systemStore.cachedAt).toLocaleString()}
          </span>
        {/if}
        <button
          type="button"
          class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          onclick={() => systemStore.refresh().catch((e) => console.error("Failed to refresh hardware info:", e))}
          disabled={systemLoading || loadingStateStore.systemInfoRefreshing}
          aria-label="Refresh hardware info"
          use:tooltip={"Refresh hardware info"}
        >
          <Icon icon="mdi:refresh" width="18" class={loadingStateStore.systemInfoRefreshing ? "animate-spin" : ""} />
        </button>
      </div>
    </div>
    <div class="divide-y divide-border rounded-lg border border-border bg-card">
      {#if systemLoading || !hw}
        <div class="space-y-2 p-3">{@render skeleton(4)}</div>
      {:else}
        {#each hardware as row, i (`${row.label}-${i}`)}
          <div class="flex items-start gap-3 px-3 py-2.5">
            <Icon icon={row.icon} width="18" class="mt-0.5 shrink-0 text-foreground-muted" />
            <div class="w-24 shrink-0 text-xs text-foreground-muted max-sm:hidden">{row.label}</div>
            <div class="min-w-0 flex-1">
              <div class="text-xs text-foreground-muted sm:hidden">{row.label}</div>
              <div class="text-[13px] font-medium wrap-break-word select-text">{row.value}</div>
              {#if row.detail}
                <div class="text-xs wrap-break-word text-foreground-muted select-text">{row.detail}</div>
              {/if}
            </div>
            {#if row.status}
              <span class="shrink-0 text-xs font-medium {row.status.tone}">{row.status.text}</span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </section>
</PageLayout>
