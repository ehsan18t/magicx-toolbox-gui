<script lang="ts" module>
  const SKELETON_WIDTHS = [90, 75, 60, 45, 30, 15];
</script>

<script lang="ts">
  import { textIfCut, tooltip } from "$lib/attachments/tooltip.svelte";
  import { type IconName, type TextTone, TONE_TEXT } from "$lib/design";
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { Button, Callout, Card, IconButton, Meter, rowButton, SectionCard, Skeleton } from "$lib/components/ui";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { systemStore } from "$lib/stores/system.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { SEP } from "$lib/utils/format";
  import { formatDate } from "$lib/utils/time";
  import { systemInfoRows, type HardwareRow } from "$lib/utils/systemInfoRows";
  import { CHECKING } from "$lib/utils/presentation";
  import { isComplete, tallies } from "$lib/utils/tweakPresentation";

  interface Tile {
    label: string;
    value: number;
    sub: string;
    icon: IconName;
    tone: TextTone;
    onclick?: () => void;
    meter?: { value: number; max: number };
  }

  const info = $derived(systemStore.info);
  const systemLoading = $derived(systemStore.isLoading);
  const rows = $derived(info ? systemInfoRows(info) : null);

  const stats = $derived(tallies(tweaksStore.list));

  // "All verified" only once every state is read: loading and unknown are not verified.
  const attentionTile = $derived.by((): Pick<Tile, "sub" | "tone" | "onclick"> => {
    if (stats.attention) {
      const [first, ...rest] = categoriesStore.withAttention;
      const more = rest.length;
      return {
        sub: `in ${categoriesStore.name(first)}${more ? ` and ${more} more` : ""}`,
        tone: "error",
        onclick: () => navigationStore.navigateToAttention(first),
      };
    }
    if (stats.byState.loading || tweaksStore.isLoading) return { sub: CHECKING.label, tone: "neutral" };
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
      meter: stats.total ? { value: stats.applied, max: stats.total } : undefined,
    },
    { label: "Needs attention", value: stats.attention, icon: "mdi:alert-circle", ...attentionTile },
    {
      label: "Snapshots",
      value: stats.withSnapshot,
      sub: "restorable",
      icon: "mdi:history",
      tone: "neutral",
      onclick: () => navigationStore.navigateToPage("snapshots"),
    },
    {
      label: "Ready to apply",
      value: pendingChangesStore.count,
      sub: pendingChangesStore.count ? "staged changes" : "nothing staged",
      icon: "mdi:arrow-right",
      tone: pendingChangesStore.count ? "warning" : "neutral",
    },
  ]);
</script>

{#snippet pcList(list: HardwareRow[], columns: boolean)}
  <dl class="m-0 grid animate-fade-in p-1 {columns ? '@min-overview-split:grid-cols-2' : ''}">
    {#each list as row, i (`${row.label}-${i}`)}
      <div class="grid grid-cols-icon-label-value items-baseline gap-x-2.5 px-2 py-1.5">
        <Icon icon={row.icon} size="sm" class="self-center text-foreground-muted" />
        <dt class="truncate text-xs text-foreground-muted">{row.label}</dt>
        <dd class="m-0 min-w-0 text-ui wrap-break-word select-text">
          <span class="font-medium">{row.value}</span>
          {#if row.detail}<span class="text-xs text-foreground-muted">{SEP}{row.detail}</span>{/if}
          {#if row.status}
            <span class="text-xs font-medium {TONE_TEXT[row.status.tone]}">{SEP}{row.status.text}</span>
          {/if}
        </dd>
      </div>
    {/each}
  </dl>
{/snippet}

<PageLayout
  title="Overview"
  description={info
    ? `${info.computer_name}${SEP}${info.username}${elevationStore.runningAs ? ` (${elevationStore.runningAs})` : ""}`
    : "Your PC at a glance"}
>
  <Card as="section" class="grid grid-cols-2 gap-px overflow-hidden bg-border sm:grid-cols-4" aria-label="Your tweaks">
    {#each tiles as t (t.label)}
      {#snippet tileBody()}
        <span class="flex items-center gap-1.5 text-xs text-foreground-muted">
          <Icon icon={t.icon} size="xs" class="shrink-0 {TONE_TEXT[t.tone]}" />
          {t.label}
        </span>
        <span class="mt-1 flex w-full min-w-0 items-baseline gap-1.5">
          <span class="font-display text-xl leading-none font-semibold tabular-nums">{t.value}</span>
          <span class="truncate text-xs text-foreground-muted" {@attach tooltip(() => textIfCut(t.sub))}>{t.sub}</span>
        </span>
        {#if t.meter}
          <Meter {...t.meter} label={t.label} class="mt-2 w-full" />
        {/if}
      {/snippet}
      {#if t.onclick}
        <button
          type="button"
          class={rowButton({ radius: "none", class: "flex min-w-0 flex-col items-start bg-card px-3 py-2.5" })}
          onclick={t.onclick}
        >
          {@render tileBody()}
        </button>
      {:else}
        <div class="flex min-w-0 flex-col items-start bg-card px-3 py-2.5">{@render tileBody()}</div>
      {/if}
    {/each}
  </Card>

  <div class="@container">
    <div class="grid items-start gap-3 @min-overview-split:grid-cols-2">
      <SectionCard title="Categories">
        <ul class="m-0 list-none p-1">
          {#each categoriesStore.list as category (category.id)}
            {@const s = categoriesStore.stats[category.id]}
            <li>
              <button
                type="button"
                class={rowButton({
                  class: "grid w-full grid-cols-icon-label-meter-value items-center gap-x-2.5 px-2 py-2",
                })}
                onclick={() =>
                  s.attention
                    ? navigationStore.navigateToAttention(category.id)
                    : navigationStore.navigateToCategory(category.id)}
                aria-label="{category.name}: {s.applied} of {s.total} applied{s.attention
                  ? `, ${s.attention} need attention`
                  : ''}"
              >
                <Icon icon={category.icon} size="md" class="text-accent" />
                <span class="flex min-w-0 items-center gap-1.5 text-ui font-medium">
                  <span class="truncate">{category.name}</span>
                  {#if s.attention}
                    <Icon icon="mdi:alert-circle" size="xs" class="shrink-0 text-error" />
                  {/if}
                </span>
                <Meter value={s.applied} max={s.total} label={category.name} />
                <span
                  class={["text-right text-xs tabular-nums", isComplete(s) ? "text-success" : "text-foreground-muted"]}
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
          <IconButton
            icon="mdi:refresh"
            size="sm"
            label="Refresh system info"
            tooltip={systemStore.cachedAt && !systemLoading
              ? `Updated ${formatDate(systemStore.cachedAt, { time: "seconds" })}. Select to refresh.`
              : "Refresh system info"}
            loading={systemStore.isRefreshing}
            disabled={systemLoading}
            onclick={() => systemStore.refresh()}
          />
        {/snippet}
        {#if rows && !systemLoading}
          {@render pcList(rows.summary, false)}
        {:else if systemStore.loadError && !systemLoading}
          <Callout tone="error" icon="mdi:alert-circle" role="alert" class="m-3">
            <div class="min-w-0 flex-1">
              <p class="m-0 text-ui">Could not read this PC's details. {systemStore.loadError}</p>
              <Button size="sm" icon="mdi:refresh" class="mt-2" onclick={() => systemStore.load()}>Retry</Button>
            </div>
          </Callout>
        {:else}
          <div class="space-y-2 p-3">
            {#each SKELETON_WIDTHS as width (width)}
              <Skeleton class="h-4" style="width: {width}%" />
            {/each}
          </div>
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
