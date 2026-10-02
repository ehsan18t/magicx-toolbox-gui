<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { AppRow, TweakRow } from "$lib/components/tweaks";
  import { EmptyState, FilterChips, SearchInput, SkeletonCard } from "$lib/components/ui";
  import type { FilterChip } from "$lib/components/ui/FilterChips.svelte";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { batchRevertTweaks, loadingStateStore, loadingStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { restartAsAdmin } from "$lib/utils/elevation";
  import { canRestore, matchesQuery, restoreMessage } from "$lib/utils/tweakPresentation";
  import { untrack } from "svelte";

  interface Props {
    tab: TabDefinition;
  }

  let { tab }: Props = $props();

  type Filter = "all" | "applied" | "not_applied" | "attention";

  let searchQuery = $state("");
  let filter = $state<Filter>(navigationStore.takeAttentionFilter() ? "attention" : "all");
  let showRevertAllDialog = $state(false);

  const categoryTweaks = $derived(tweaksStore.list.filter((t) => t.definition.category_id === tab.id));
  const appliedCount = $derived(categoryTweaks.filter((t) => t.status.is_applied).length);
  const attentionCount = $derived(categoryTweaks.filter((t) => t.status.attention).length);
  const needsAdminCount = $derived(
    categoryTweaks.filter((t) => t.definition.availability.state === "needs_elevation").length,
  );
  const tweaksWithSnapshots = $derived(categoryTweaks.filter(canRestore));

  const filters = $derived<FilterChip<Filter>[]>([
    { id: "all", label: "All", count: categoryTweaks.length },
    { id: "applied", label: "Applied", count: appliedCount },
    { id: "not_applied", label: "Not applied", count: categoryTweaks.length - appliedCount },
    ...(attentionCount > 0
      ? [{ id: "attention" as const, label: "Needs attention", count: attentionCount, tone: "error" as const }]
      : []),
  ]);
  // Fall back to All once the last item resolves, so a later one cannot silently narrow the list.
  $effect(() => {
    if (attentionCount === 0 && untrack(() => filter) === "attention") filter = "all";
  });

  function matchesFilter(t: TweakWithStatus): boolean {
    if (filter === "applied") return t.status.is_applied;
    if (filter === "not_applied") return !t.status.is_applied;
    if (filter === "attention") return t.status.attention !== null;
    return true;
  }

  const query = $derived(searchQuery.trim().toLowerCase());
  const filteredTweaks = $derived(categoryTweaks.filter((t) => matchesFilter(t) && matchesQuery(t, query)));

  const categoryApps = $derived(appsStore.byCategory[tab.id] ?? []);
  const filteredApps = $derived(
    filter !== "all"
      ? []
      : categoryApps.filter(
          (a) => !query || a.name.toLowerCase().includes(query) || a.description.toLowerCase().includes(query),
        ),
  );
  const installedAppCount = $derived(
    categoryApps.filter((a) => appsStore.status(a.id)?.presence.state === "installed").length,
  );

  async function handleRestoreSnapshots() {
    showRevertAllDialog = false;
    await batchRevertTweaks(tweaksWithSnapshots.map((t) => t.definition.id));
  }

  function resetFilters() {
    searchQuery = "";
    filter = "all";
  }
</script>

<PageLayout title={tab.name} description={tab.description}>
  {#snippet aside()}
    {#if categoryTweaks.length > 0}
      <div class="flex w-44 shrink-0 flex-col gap-1.5">
        <div class="flex items-baseline justify-between text-xs">
          <span class="text-foreground-muted">Applied</span>
          <span class="font-semibold tabular-nums">{appliedCount} of {categoryTweaks.length}</span>
        </div>
        <div class="h-1 overflow-hidden rounded-full bg-muted">
          <div
            class="h-full rounded-full bg-accent transition-[width] duration-300"
            style="width: {(appliedCount / categoryTweaks.length) * 100}%"
          ></div>
        </div>
      </div>
    {/if}
  {/snippet}

  {#snippet toolbar()}
    <SearchInput
      value={searchQuery}
      placeholder="Filter this category"
      class="min-w-48 flex-1 sm:max-w-80"
      onchange={(v) => (searchQuery = v)}
    />
    <FilterChips options={filters} value={filter} onchange={(v) => (filter = v)} />
    {#if tweaksWithSnapshots.length > 0}
      <button
        type="button"
        class="ml-auto inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
        disabled={loadingStore.busy}
        onclick={() => (showRevertAllDialog = true)}
      >
        <Icon icon="mdi:history" width="16" />
        Restore all
        <span class="text-xs text-foreground-subtle tabular-nums">{tweaksWithSnapshots.length}</span>
      </button>
    {/if}
  {/snippet}

  {#if needsAdminCount > 0}
    <div
      class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-warning/30 bg-warning/8 px-3 py-2.5"
    >
      <Icon icon="mdi:shield-lock-outline" width="18" class="shrink-0 text-warning" />
      <p class="m-0 min-w-0 flex-1 text-[13px]">
        <span class="font-semibold">
          {needsAdminCount === 1 ? "1 tweak here needs" : `${needsAdminCount} tweaks here need`} administrator rights.
        </span>
        <span class="text-foreground-muted">They stay read-only until you restart as administrator.</span>
      </p>
      <button
        type="button"
        class="h-8 cursor-pointer rounded-md bg-accent px-3 text-[13px] font-semibold text-accent-foreground hover:bg-accent-hover"
        onclick={restartAsAdmin}
      >
        Restart as admin
      </button>
    </div>
  {/if}

  {#if loadingStateStore.tweaksLoading && categoryTweaks.length === 0}
    <SkeletonCard />
  {:else if filteredTweaks.length === 0 && filteredApps.length === 0}
    <EmptyState
      icon={query ? "mdi:file-search-outline" : "mdi:package-variant"}
      title={query || filter !== "all" ? "Nothing matches" : "No tweaks available"}
      description={query
        ? `Nothing in ${tab.name} matches "${searchQuery}"`
        : filter !== "all"
          ? "No tweaks in this category match the selected filter"
          : "This category has no tweaks for your system"}
      actionText={query || filter !== "all" ? "Show all" : undefined}
      onaction={resetFilters}
    />
  {:else}
    {#if filteredTweaks.length > 0}
      <div class="flex flex-col gap-2">
        {#each filteredTweaks as tweak (tweak.definition.id)}
          <TweakRow {tweak} />
        {/each}
      </div>
    {/if}

    {#if filteredApps.length > 0}
      <section aria-labelledby="apps-heading-{tab.id}" class="mt-4 flex flex-col gap-2">
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

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore {tab.name}"
  message={restoreMessage(tweaksWithSnapshots.length)}
  confirmText="Restore"
  variant="danger"
  onconfirm={handleRestoreSnapshots}
  oncancel={() => (showRevertAllDialog = false)}
/>
