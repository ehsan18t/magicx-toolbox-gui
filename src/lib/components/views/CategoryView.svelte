<script lang="ts">
  import { NoMatches, PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { AppRow, RestoreAllButton, TweakRow } from "$lib/components/tweaks";
  import { Button, Callout, EmptyState, ICON_SIZE, Meter, SkeletonList } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { plural } from "$lib/utils/format";
  import { canRestore, tallies } from "$lib/utils/tweakPresentation";
  import { untrack } from "svelte";

  let { tab }: { tab: TabDefinition } = $props();

  let attentionOnly = $state(navigationStore.takeAttentionFilter());

  const categoryTweaks = $derived(tweaksStore.byCategory[tab.id] ?? []);
  const stats = $derived(tallies(categoryTweaks));
  const restorable = $derived(categoryTweaks.filter(canRestore));

  // Off once the last item resolves, so a later one cannot silently narrow the list.
  $effect(() => {
    if (stats.attention === 0 && untrack(() => attentionOnly)) attentionOnly = false;
  });

  const query = $derived(pageFilterStore.query.trim());
  const filteredTweaks = $derived(
    categoryTweaks.filter(
      (t) => (!attentionOnly || t.status.attention !== null) && pageFilterStore.passes(t.definition.id),
    ),
  );

  const categoryApps = $derived(appsStore.byCategory[tab.id] ?? []);
  const filteredApps = $derived(attentionOnly ? [] : categoryApps.filter((a) => pageFilterStore.passes(a.id)));
  const installedAppCount = $derived(
    categoryApps.filter((a) => appsStore.status(a.id)?.presence.state === "installed").length,
  );
</script>

<PageLayout title={tab.name} description={tab.description}>
  {#snippet aside()}
    <div class="flex w-full flex-wrap items-center gap-x-4 gap-y-2">
      {#if stats.attention > 0}
        <button
          type="button"
          class={[
            "inline-flex h-8 animate-pop-in cursor-pointer items-center gap-1.5 rounded-full border px-3 text-ui font-medium",
            attentionOnly
              ? "border-error bg-error text-background"
              : "border-error/40 bg-error/10 text-error hover:bg-error/15",
          ]}
          aria-pressed={attentionOnly}
          onclick={() => (attentionOnly = !attentionOnly)}
        >
          <Icon icon="mdi:alert-circle" width={ICON_SIZE.md} />
          {plural(stats.attention, "needs", "need")} attention
        </button>
      {/if}
      {#if stats.total > 0}
        <div class="flex w-44 shrink-0 flex-col gap-1.5">
          <div class="flex items-baseline justify-between text-xs">
            <span class="text-foreground-muted">Applied</span>
            <span class="font-semibold tabular-nums">{stats.applied} of {stats.total}</span>
          </div>
          <Meter value={stats.applied} max={stats.total} label="Applied" />
        </div>
      {/if}
      {#if restorable.length > 0}
        <RestoreAllButton title="Restore {tab.name}?" tweaks={restorable} class="ml-auto" />
      {/if}
    </div>
  {/snippet}

  {#if stats.needsAdmin > 0}
    <Callout tone="warning" class="flex animate-fade-in flex-wrap items-center gap-y-2">
      <Icon icon="mdi:shield-lock-outline" width={ICON_SIZE.lg} class="shrink-0 text-warning" />
      <p class="m-0 min-w-0 flex-1 text-ui">
        <span class="font-semibold">
          {plural(stats.needsAdmin, "tweak here needs", "tweaks here need")} administrator rights.
        </span>
        <span class="text-foreground-muted">They stay read-only until you restart as administrator.</span>
      </p>
      <Button variant="primary" class="font-semibold" onclick={elevationStore.restartAsAdmin}>Restart as admin</Button>
    </Callout>
  {/if}

  {#if tweaksStore.isLoading && categoryTweaks.length === 0}
    <SkeletonList />
  {:else if filteredTweaks.length === 0 && filteredApps.length === 0}
    {#if query}
      <NoMatches description={`Nothing in ${tab.name} matches "${query}"`} />
    {:else if attentionOnly}
      <EmptyState
        icon="mdi:check-circle-outline"
        title="Nothing needs attention"
        description="No tweak in this category needs attention."
        action={{ label: "Show all", onclick: () => (attentionOnly = false) }}
      />
    {:else}
      <EmptyState
        icon="mdi:package-variant"
        title="No tweaks available"
        description="This category has no tweaks for your system"
      />
    {/if}
  {:else}
    {#if filteredTweaks.length > 0}
      <div class="flex animate-fade-in flex-col gap-2">
        {#each filteredTweaks as tweak (tweak.definition.id)}
          <TweakRow {tweak} />
        {/each}
      </div>
    {/if}

    {#if filteredApps.length > 0}
      <section aria-labelledby="apps-heading-{tab.id}" class="mt-4 flex animate-fade-in flex-col gap-2">
        <h2 id="apps-heading-{tab.id}" class="m-0 flex items-baseline gap-2 text-base font-semibold">
          Apps
          <span class="text-xs font-normal text-foreground-muted">{installedAppCount} installed</span>
        </h2>
        {#each filteredApps as app (app.id)}
          <AppRow {app} />
        {/each}
      </section>
    {/if}
  {/if}
</PageLayout>
