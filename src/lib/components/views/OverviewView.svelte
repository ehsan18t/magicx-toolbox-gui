<script lang="ts" module>
  const SKELETON_WIDTHS = [90, 75, 60, 45, 30, 15];
</script>

<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { PageLayout } from "$lib/components/layout";
  import { Icon, type IconName } from "$lib/components/shared";
  import { ICON_SIZE, Meter, SectionCard, TONE_TEXT, type TextTone } from "$lib/components/ui";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { systemStore } from "$lib/stores/system.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { formatDate } from "$lib/utils/format";
  import { systemInfoRows, type HardwareRow } from "$lib/utils/systemInfoRows";
  import { tallies } from "$lib/utils/tweakPresentation";

  interface Tile {
    label: string;
    value: number;
    sub: string;
    icon: IconName;
    tone: TextTone;
    onclick?: () => void;
    /** Percent, drawn as a meter under the value. */
    progress?: number;
  }

  const info = $derived(systemStore.info);
  const systemLoading = $derived(systemStore.isLoading);
  const rows = $derived(info ? systemInfoRows(info) : null);

  const stats = $derived(tallies(tweaksStore.list));
  const attentionCategories = $derived(
    new Set(tweaksStore.list.filter((t) => t.status.attention).map((t) => t.definition.categoryId)),
  );

  // "All verified" only once every state is read: loading and unknown are not verified.
  const attentionTile = $derived.by((): Pick<Tile, "sub" | "tone" | "onclick"> => {
    if (stats.attention) {
      const [first] = attentionCategories;
      const more = attentionCategories.size - 1;
      return {
        sub: `in ${categoriesStore.name(first)}${more ? ` and ${more} more` : ""}`,
        tone: "error",
        onclick: () => navigationStore.navigateToAttention(first),
      };
    }
    if (stats.byState.loading || tweaksStore.isLoading) return { sub: "Checking…", tone: "neutral" };
    if (stats.byState.unknown) return { sub: `${stats.byState.unknown} could not be read`, tone: "warning" };
    return { sub: "All verified", tone: "success" };
  });

  const tiles = $derived<Tile[]>([
    {
      label: "Applied",
      value: stats.applied,
      sub: `of ${stats.total} tweaks`,
      icon: "mdi:check-circle",
      tone: "accent",
      progress: stats.total ? (stats.applied / stats.total) * 100 : undefined,
    },
    { label: "Needs attention", value: stats.attention, icon: "mdi:alert-circle", ...attentionTile },
    {
      label: "Snapshots",
      value: stats.withSnapshot,
      sub: "restorable",
      icon: "mdi:history",
      tone: "neutral",
      onclick: () => navigationStore.navigateToTab("snapshots"),
    },
    {
      label: "Ready to apply",
      value: pendingChangesStore.count,
      sub: pendingChangesStore.count ? "staged changes" : "nothing staged",
      icon: "mdi:arrow-right",
      tone: pendingChangesStore.count ? "warning" : "neutral",
    },
  ]);

  // The store logs the failure.
  const refresh = () => systemStore.refresh().catch(() => {});
</script>

{#snippet pcList(list: HardwareRow[], columns: boolean)}
  <dl class="m-0 grid animate-fade-in p-1 {columns ? '@min-overview-split:grid-cols-2' : ''}">
    {#each list as row, i (`${row.label}-${i}`)}
      <div class="grid grid-cols-icon-label-value items-baseline gap-x-2.5 px-2 py-1.5">
        <Icon icon={row.icon} width="15" class="self-center text-foreground-muted" />
        <dt class="truncate text-xs text-foreground-muted">{row.label}</dt>
        <dd class="m-0 min-w-0 text-ui wrap-break-word select-text">
          <span class="font-medium">{row.value}</span>
          {#if row.detail}<span class="text-xs text-foreground-muted"> · {row.detail}</span>{/if}
          {#if row.status}
            <span class="text-xs font-medium {TONE_TEXT[row.status.tone]}"> · {row.status.text}</span>
          {/if}
        </dd>
      </div>
    {/each}
  </dl>
{/snippet}

<PageLayout
  title="Overview"
  description={info
    ? `${info.computer_name} · ${info.username} (${info.is_admin ? "Administrator" : "Standard user"})`
    : "Your PC at a glance"}
>
  <section
    class="grid grid-cols-2 overflow-hidden rounded-lg border border-border bg-card *:border-border sm:grid-cols-4 sm:[&>*:not(:last-child)]:border-r max-sm:[&>*:nth-child(-n+2)]:border-b max-sm:[&>*:nth-child(odd)]:border-r"
    aria-label="Your tweaks"
  >
    {#each tiles as t (t.label)}
      {#snippet tileBody()}
        <span class="flex items-center gap-1.5 text-xs text-foreground-muted">
          <Icon icon={t.icon} width={ICON_SIZE.sm} class="shrink-0 {TONE_TEXT[t.tone]}" />
          {t.label}
        </span>
        <span class="mt-1 flex w-full min-w-0 items-baseline gap-1.5">
          <span class="font-display text-xl leading-none font-semibold tabular-nums">{t.value}</span>
          <span class="truncate text-xs text-foreground-muted" title={t.sub}>{t.sub}</span>
        </span>
        {#if t.progress !== undefined}
          <Meter value={t.progress} label={t.label} class="mt-2 w-full" />
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
    <div class="grid items-start gap-3 @min-overview-split:grid-cols-2">
      <SectionCard title="Categories">
        <ul class="m-0 list-none p-1">
          {#each categoriesStore.list as category (category.id)}
            {@const s = categoriesStore.stats[category.id]}
            <li>
              <button
                type="button"
                class="grid w-full cursor-pointer grid-cols-icon-label-meter-value items-center gap-x-2.5 rounded-md px-2 py-2 text-left hover:bg-muted"
                onclick={() =>
                  s.attention
                    ? navigationStore.navigateToAttention(category.id)
                    : navigationStore.navigateToTab(category.id)}
                aria-label="{category.name}: {s.applied} of {s.total} applied{s.attention
                  ? `, ${s.attention} need attention`
                  : ''}"
              >
                <Icon icon={category.icon} width={ICON_SIZE.md} class="text-accent" />
                <span class="flex min-w-0 items-center gap-1.5 text-ui font-medium">
                  <span class="truncate">{category.name}</span>
                  {#if s.attention}
                    <Icon icon="mdi:alert-circle" width={ICON_SIZE.sm} class="shrink-0 text-error" />
                  {/if}
                </span>
                <Meter value={s.applied} max={s.total} label={category.name} />
                <span
                  class={[
                    "text-right text-xs tabular-nums",
                    s.total > 0 && s.applied === s.total ? "text-success" : "text-foreground-muted",
                  ]}
                >
                  {s.applied}/{s.total}
                </span>
              </button>
            </li>
          {/each}
        </ul>
      </SectionCard>

      <SectionCard title="This PC">
        {#snippet actions()}
          <button
            type="button"
            class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            onclick={refresh}
            disabled={systemLoading || systemStore.isRefreshing}
            aria-label="Refresh system info"
            use:tooltip={systemStore.cachedAt && !systemLoading
              ? `Updated ${formatDate(systemStore.cachedAt, { time: "seconds" })}. Select to refresh.`
              : "Refresh system info"}
          >
            <Icon icon="mdi:refresh" width={ICON_SIZE.md} class={systemStore.isRefreshing ? "animate-spin" : ""} />
          </button>
        {/snippet}
        {#if systemLoading || !rows}
          <div class="space-y-2 p-3">
            {#each SKELETON_WIDTHS as width (width)}
              <div class="h-4 animate-pulse rounded bg-muted" style:width="{width}%"></div>
            {/each}
          </div>
        {:else}
          {@render pcList(rows.summary, false)}
        {/if}
      </SectionCard>
    </div>

    {#if rows && rows.devices.length > 0}
      <SectionCard title="Devices" class="mt-3 animate-fade-in">
        {@render pcList(rows.devices, true)}
      </SectionCard>
    {/if}
  </div>
</PageLayout>
