<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { AppRow, TweakRow } from "$lib/components/tweaks";
  import { EmptyState, SkeletonCard } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweakOps } from "$lib/stores/tweakOps.svelte";
  import { restoreTweaks } from "$lib/stores/tweaksActions.svelte";
  import { initStatus, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { canRestore, restoreMessage } from "$lib/utils/tweakPresentation";
  import { untrack } from "svelte";

  interface Props {
    tab: TabDefinition;
  }

  let { tab }: Props = $props();

  let attentionOnly = $state(navigationStore.takeAttentionFilter());

  const categoryTweaks = $derived(tweaksStore.list.filter((t) => t.definition.categoryId === tab.id));
  const appliedCount = $derived(categoryTweaks.filter((t) => t.status.state === "active").length);
  const attentionCount = $derived(categoryTweaks.filter((t) => t.status.attention).length);
  const needsAdminCount = $derived(
    categoryTweaks.filter((t) => t.definition.availability.state === "needs_elevation").length,
  );
  const restorable = $derived(categoryTweaks.filter(canRestore));

  // Off once the last item resolves, so a later one cannot silently narrow the list.
  $effect(() => {
    if (attentionCount === 0 && untrack(() => attentionOnly)) attentionOnly = false;
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

  async function restoreAll() {
    const ids = restorable.map((t) => t.definition.id);
    if (
      await confirmStore.ask({
        title: `Restore ${tab.name}?`,
        message: restoreMessage(ids.length),
        confirmText: "Restore",
        variant: "danger",
      })
    )
      await restoreTweaks(ids);
  }
</script>

<PageLayout title={tab.name} description={tab.description}>
  {#snippet aside()}
    <div class="flex w-full flex-wrap items-center gap-x-4 gap-y-2">
      {#if attentionCount > 0}
        <button
          type="button"
          class="inline-flex h-8 animate-pop-in cursor-pointer items-center gap-1.5 rounded-full border px-3 text-ui font-medium {attentionOnly
            ? 'border-error bg-error text-background'
            : 'border-error/40 bg-error/10 text-error hover:bg-error/15'}"
          aria-pressed={attentionOnly}
          onclick={() => (attentionOnly = !attentionOnly)}
        >
          <Icon icon="mdi:alert-circle" width="16" />
          {attentionCount} need{attentionCount === 1 ? "s" : ""} attention
        </button>
      {/if}
      {#if categoryTweaks.length > 0}
        <div class="flex w-44 shrink-0 flex-col gap-1.5">
          <div class="flex items-baseline justify-between text-xs">
            <span class="text-foreground-muted">Applied</span>
            <span class="font-semibold tabular-nums">{appliedCount} of {categoryTweaks.length}</span>
          </div>
          <div class="h-1 overflow-hidden rounded-full bg-muted">
            <div
              class="h-full rounded-full bg-accent transition-[width] duration-slower ease-out"
              style="width: {(appliedCount / categoryTweaks.length) * 100}%"
            ></div>
          </div>
        </div>
      {/if}
      {#if restorable.length > 0}
        <button
          type="button"
          class="ml-auto inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-ui font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
          disabled={tweakOps.isBusy}
          onclick={restoreAll}
        >
          <Icon icon="mdi:history" width="16" />
          Restore all
          <span class="text-xs text-foreground-subtle tabular-nums">{restorable.length}</span>
        </button>
      {/if}
    </div>
  {/snippet}

  {#if needsAdminCount > 0}
    <div
      class="flex animate-fade-in flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-warning/30 bg-warning/8 px-3 py-2.5"
    >
      <Icon icon="mdi:shield-lock-outline" width="18" class="shrink-0 text-warning" />
      <p class="m-0 min-w-0 flex-1 text-ui">
        <span class="font-semibold">
          {needsAdminCount === 1 ? "1 tweak here needs" : `${needsAdminCount} tweaks here need`} administrator rights.
        </span>
        <span class="text-foreground-muted">They stay read-only until you restart as administrator.</span>
      </p>
      <button
        type="button"
        class="h-8 cursor-pointer rounded-md bg-accent px-3 text-ui font-semibold text-accent-foreground hover:bg-accent-hover"
        onclick={elevationStore.restartAsAdmin}
      >
        Restart as admin
      </button>
    </div>
  {/if}

  {#if initStatus.isLoadingTweaks && categoryTweaks.length === 0}
    <SkeletonCard />
  {:else if filteredTweaks.length === 0 && filteredApps.length === 0}
    {#if query}
      <EmptyState
        icon="mdi:file-search-outline"
        title="Nothing matches"
        description={`Nothing in ${tab.name} matches "${query}"`}
        actionText="Search everywhere"
        onaction={() => pageFilterStore.searchEverywhere()}
      />
    {:else if attentionOnly}
      <EmptyState
        icon="mdi:check-circle-outline"
        title="Nothing needs attention"
        description="No tweak in this category needs attention."
        actionText="Show all"
        onaction={() => (attentionOnly = false)}
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
