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
  const attentionTweaks = $derived(tweaksStore.list.filter((t) => t.status.attention));
  const attention = $derived(attentionTweaks.length);
  const checking = $derived(tweaksStore.list.filter((t) => t.status.state === "loading").length);
  const unknown = $derived(tweaksStore.list.filter((t) => t.status.state === "unknown").length);
  const attentionCategories = $derived(new Set(attentionTweaks.map((t) => t.definition.category_id)));

  // "All verified" only once every state is read: loading and unknown are not verified.
  const attentionTile = $derived.by(() => {
    if (attention) {
      const [first] = attentionCategories;
      const single = attentionCategories.size === 1;
      return {
        sub: `in ${categoriesStore.getName(first)}${single ? "" : ` and ${attentionCategories.size - 1} more`}`,
        tone: "text-error",
        onclick: () => navigationStore.navigateToAttention(first),
      };
    }
    if (checking || loadingStateStore.tweaksLoading)
      return { sub: "Checking…", tone: "text-foreground-muted", onclick: null };
    if (unknown) return { sub: `${unknown} could not be read`, tone: "text-warning", onclick: null };
    return { sub: "All verified", tone: "text-success", onclick: null };
  });
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
    /** Listed under Devices rather than in the PC summary. */
    device?: boolean;
  }

  const pcRows = $derived.by((): HardwareRow[] => {
    if (!info || !hw) return [];
    const rows: HardwareRow[] = [
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
        device: true,
        value: gpu.name,
        detail: [
          gpu.memory_gb > 0 ? `${gpu.memory_gb} GB` : "Shared memory",
          gpu.driver_version && `Driver ${gpu.driver_version}`,
          hw.monitors.length === 0 && gpu.refresh_rate > 0 && `${gpu.refresh_rate} Hz`,
        ]
          .filter(Boolean)
          .join(" · "),
      }),
    );
    hw.monitors.forEach((m, i) =>
      rows.push({
        icon: "mdi:monitor",
        label: hw.monitors.length > 1 ? `Display ${i + 1}` : "Display",
        device: true,
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
        device: true,
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
        device: true,
        value: n.name,
        detail: [n.ip_address || "Not connected", n.mac_address].filter(Boolean).join(" · "),
      }),
    );
    return rows;
  });
  const summaryRows = $derived(pcRows.filter((r) => !r.device));
  const deviceRows = $derived(pcRows.filter((r) => r.device));

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
      icon: "mdi:alert-circle",
      ...attentionTile,
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

{#snippet pcList(rows: HardwareRow[], columns: boolean)}
  <dl class="m-0 grid animate-fade-in p-1 {columns ? '@min-overview:grid-cols-2' : ''}">
    {#each rows as row, i (`${row.label}-${i}`)}
      <div class="grid grid-cols-hardware items-baseline gap-x-2.5 px-2 py-1.5">
        <Icon icon={row.icon} width="15" class="self-center text-foreground-muted" />
        <dt class="truncate text-xs text-foreground-muted">{row.label}</dt>
        <dd class="m-0 min-w-0 text-ui wrap-break-word select-text">
          <span class="font-medium">{row.value}</span>
          {#if row.detail}<span class="text-xs text-foreground-muted"> · {row.detail}</span>{/if}
          {#if row.status}<span class="text-xs font-medium {row.status.tone}"> · {row.status.text}</span>{/if}
        </dd>
      </div>
    {/each}
  </dl>
{/snippet}

{#snippet skeleton(lines: number)}
  {#each Array.from({ length: lines }, (_, i) => i) as i (i)}
    <div class="h-4 animate-pulse rounded bg-muted" style="width: {90 - i * 15}%"></div>
  {/each}
{/snippet}

<PageLayout
  title="Overview"
  description={info
    ? `${info.computer_name} · ${info.username} (${info.is_admin ? "Administrator" : "Standard user"})`
    : "Your PC at a glance"}
>
  <section
    class="grid grid-cols-2 overflow-hidden rounded-lg border border-border bg-card sm:grid-cols-4 [&>*]:border-border sm:[&>*:not(:last-child)]:border-r max-sm:[&>*:nth-child(-n+2)]:border-b max-sm:[&>*:nth-child(odd)]:border-r"
    aria-label="Your tweaks"
  >
    {#each tiles as t, i (t.label)}
      {#snippet tileBody()}
        <span class="flex items-center gap-1.5 text-xs text-foreground-muted">
          <Icon icon={t.icon} width="14" class="shrink-0 {t.tone}" />
          {t.label}
        </span>
        <span class="mt-1 flex w-full min-w-0 items-baseline gap-1.5">
          <span class="font-display text-xl leading-none font-semibold tabular-nums">{t.value}</span>
          <span class="truncate text-xs text-foreground-muted" title={t.sub}>{t.sub}</span>
        </span>
        {#if i === 0 && tweaksStore.list.length > 0}
          <span class="mt-2 block h-1 w-full overflow-hidden rounded-full bg-muted">
            <span
              class="block h-full rounded-full bg-accent transition-[width] duration-slower ease-out"
              style="width: {(applied / tweaksStore.list.length) * 100}%"
            ></span>
          </span>
        {/if}
      {/snippet}
      {#if t.onclick}
        <button
          type="button"
          class="flex min-w-0 cursor-pointer flex-col items-start px-3 py-2.5 text-left hover:bg-muted"
          onclick={t.onclick}
        >
          {@render tileBody()}
        </button>
      {:else}
        <div class="flex min-w-0 flex-col items-start px-3 py-2.5">{@render tileBody()}</div>
      {/if}
    {/each}
  </section>

  <div class="@container">
    <div class="grid items-start gap-3 @min-overview:grid-cols-2">
      <section class="overflow-hidden rounded-lg border border-border bg-card" aria-labelledby="overview-categories">
        <h2 id="overview-categories" class="m-0 border-b border-border px-3 py-2 text-ui font-semibold">Categories</h2>
        <ul class="m-0 list-none p-1">
          {#each categoriesStore.list as category (category.id)}
            {@const s = categoryStats[category.id]}
            {@const progress = s?.total ? (s.applied / s.total) * 100 : 0}
            <li>
              <button
                type="button"
                class="grid w-full cursor-pointer grid-cols-category-progress items-center gap-x-2.5 rounded-md px-2 py-2 text-left hover:bg-muted"
                onclick={() =>
                  s?.attention
                    ? navigationStore.navigateToAttention(category.id)
                    : navigationStore.navigateToCategory(category.id)}
                aria-label="{category.name}: {s?.applied ?? 0} of {s?.total ?? 0} applied{s?.attention
                  ? `, ${s.attention} need attention`
                  : ''}"
              >
                <Icon icon={category.icon || "mdi:folder"} width="16" class="text-accent" />
                <span class="flex min-w-0 items-center gap-1.5 text-ui font-medium">
                  <span class="truncate">{category.name}</span>
                  {#if s?.attention}
                    <Icon icon="mdi:alert-circle" width="14" class="shrink-0 text-error" />
                  {/if}
                </span>
                <span class="block h-1 overflow-hidden rounded-full bg-muted">
                  <span
                    class="block h-full rounded-full bg-accent transition-[width] duration-slower ease-out"
                    style="width: {progress}%"
                  ></span>
                </span>
                <span
                  class="text-right text-xs tabular-nums {s && s.total > 0 && s.applied === s.total
                    ? 'text-success'
                    : 'text-foreground-muted'}"
                >
                  {s?.applied ?? 0}/{s?.total ?? 0}
                </span>
              </button>
            </li>
          {/each}
        </ul>
      </section>

      <section class="overflow-hidden rounded-lg border border-border bg-card" aria-labelledby="overview-pc">
        <div class="flex items-center justify-between gap-3 border-b border-border py-1 pr-1 pl-3">
          <h2 id="overview-pc" class="m-0 text-ui font-semibold">This PC</h2>
          <button
            type="button"
            class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => systemStore.refresh().catch((e) => console.error("Failed to refresh hardware info:", e))}
            disabled={systemLoading || loadingStateStore.systemInfoRefreshing}
            aria-label="Refresh system info"
            use:tooltip={systemStore.cachedAt && !systemLoading
              ? `Updated ${new Date(systemStore.cachedAt).toLocaleString()}. Select to refresh.`
              : "Refresh system info"}
          >
            <Icon icon="mdi:refresh" width="16" class={loadingStateStore.systemInfoRefreshing ? "animate-spin" : ""} />
          </button>
        </div>
        {#if systemLoading || !hw}
          <div class="space-y-2 p-3">{@render skeleton(6)}</div>
        {:else}
          {@render pcList(summaryRows, false)}
        {/if}
      </section>
    </div>

    {#if deviceRows.length > 0}
      <section
        class="mt-3 animate-fade-in overflow-hidden rounded-lg border border-border bg-card"
        aria-labelledby="overview-devices"
      >
        <h2 id="overview-devices" class="m-0 border-b border-border px-3 py-2 text-ui font-semibold">Devices</h2>
        {@render pcList(deviceRows, true)}
      </section>
    {/if}
  </div>
</PageLayout>
