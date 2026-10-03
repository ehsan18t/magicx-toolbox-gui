<script lang="ts">
  import { NoMatches } from "$lib/components/feedback";
  import { AppliedMeter, PageLayout } from "$lib/components/layout";
  import { AppRow, RestoreAllButton, TweakRow } from "$lib/components/items";
  import { Button, Callout, EmptyState, SkeletonList, ToggleChip } from "$lib/components/ui";
  import { HEADING } from "$lib/design";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { plural } from "$lib/utils/format";
  import { canRestore, tallies } from "$lib/utils/tweakPresentation";
  import { untrack } from "svelte";

  let { tab }: { tab: TabDefinition } = $props();

  const appsHeadingId = $props.id();

  let attentionOnly = $state(navigationStore.takeAttentionFilter());

  const categoryTweaks = $derived(tweaksStore.byCategory[tab.id] ?? []);
  const stats = $derived(tallies(categoryTweaks));
  const restorable = $derived(categoryTweaks.filter(canRestore));

  // Off once the last item resolves, so a later one cannot silently narrow the list.
  $effect(() => {
    if (stats.attention === 0 && untrack(() => attentionOnly)) attentionOnly = false;
  });

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
    {#if stats.attention > 0}
      <ToggleChip
        size="md"
        tone="error"
        variant={attentionOnly ? "solid" : "tint"}
        icon="mdi:alert-circle"
        class="animate-pop-in"
        aria-pressed={attentionOnly}
        onclick={() => (attentionOnly = !attentionOnly)}
      >
        {plural(stats.attention, "needs", "need")} attention
      </ToggleChip>
    {/if}
    {#if stats.total > 0}
      <AppliedMeter applied={stats.applied} total={stats.total} class="w-44 shrink-0" />
    {/if}
    {#if restorable.length > 0}
      <RestoreAllButton title="Restore {tab.name}?" tweaks={restorable} class="ml-auto" />
    {/if}
  {/snippet}

  {#if stats.needsAdmin > 0}
    <Callout tone="warning" icon="mdi:shield-lock-outline" class="animate-fade-in flex-wrap items-center gap-y-2">
      <p class="m-0 min-w-0 flex-1 text-ui">
        <span class="font-semibold">
          {plural(stats.needsAdmin, "tweak here needs", "tweaks here need")} administrator rights.
        </span>
        <span class="text-foreground-muted">They stay read-only until you restart as administrator.</span>
      </p>
      <Button variant="primary" loading={elevationStore.isRestarting} onclick={elevationStore.restartAsAdmin}>
        Restart as admin
      </Button>
    </Callout>
  {/if}

  {#if tweaksStore.isLoading && categoryTweaks.length === 0}
    <SkeletonList />
  {:else if filteredTweaks.length === 0 && filteredApps.length === 0}
    {#if pageFilterStore.trimmedQuery}
      <NoMatches description={`Nothing in ${tab.name} matches "${pageFilterStore.trimmedQuery}"`} />
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
        description="This category has no tweaks for your system."
      />
    {/if}
  {:else}
    {#if filteredTweaks.length > 0}
      <h2 class="sr-only">Tweaks</h2>
      <div class="flex animate-fade-in flex-col gap-2">
        {#each filteredTweaks as tweak (tweak.definition.id)}
          <TweakRow {tweak} />
        {/each}
      </div>
    {/if}

    {#if filteredApps.length > 0}
      <section aria-labelledby={appsHeadingId} class="mt-4 flex animate-fade-in flex-col gap-2">
        <h2 id={appsHeadingId} class={["m-0 flex items-baseline gap-2", HEADING.section]}>
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
