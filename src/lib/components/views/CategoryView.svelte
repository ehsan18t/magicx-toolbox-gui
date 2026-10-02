<script lang="ts">
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { AppCard, TweakRow } from "$lib/components/tweaks";
  import { EmptyState, SkeletonCard } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import type { TabDefinition } from "$lib/stores/navigation.svelte";
  import { batchRevertTweaks, loadingStateStore, loadingStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { restartAsAdmin } from "$lib/utils/elevation";

  interface Props {
    tab: TabDefinition;
  }

  let { tab }: Props = $props();

  type Filter = "all" | "applied" | "not_applied" | "attention";

  let searchQuery = $state("");
  let filter = $state<Filter>("all");
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);
  const categoryTweaks = $derived(tweaksStore.list.filter((t) => t.definition.category_id === tab.id));
  const appliedCount = $derived(categoryTweaks.filter((t) => t.status.is_applied).length);
  const attentionCount = $derived(categoryTweaks.filter((t) => t.status.attention).length);
  const needsAdminCount = $derived(
    categoryTweaks.filter((t) => t.definition.availability.state === "needs_elevation").length,
  );
  const tweaksWithSnapshots = $derived(categoryTweaks.filter((t) => t.status.has_backup));
  const isLoading = $derived(categoryTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  const filters = $derived<{ id: Filter; label: string; count: number }[]>([
    { id: "all", label: "All", count: categoryTweaks.length },
    { id: "applied", label: "Applied", count: appliedCount },
    { id: "not_applied", label: "Not applied", count: categoryTweaks.length - appliedCount },
    ...(attentionCount > 0 ? [{ id: "attention" as const, label: "Needs attention", count: attentionCount }] : []),
  ]);

  const activeFilter = $derived<Filter>(filter === "attention" && attentionCount === 0 ? "all" : filter);

  function matchesFilter(t: TweakWithStatus): boolean {
    if (activeFilter === "applied") return t.status.is_applied;
    if (activeFilter === "not_applied") return !t.status.is_applied;
    if (activeFilter === "attention") return t.status.attention !== null;
    return true;
  }

  const query = $derived(searchQuery.trim().toLowerCase());
  const filteredTweaks = $derived(
    categoryTweaks.filter(
      (t) =>
        matchesFilter(t) &&
        (!query ||
          t.definition.name.toLowerCase().includes(query) ||
          t.definition.description.toLowerCase().includes(query)),
    ),
  );

  const categoryApps = $derived(appsStore.byCategory[tab.id] ?? []);
  const filteredApps = $derived(
    activeFilter !== "all"
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
    isBatchProcessing = true;
    await batchRevertTweaks(tweaksWithSnapshots.map((t) => t.definition.id));
    isBatchProcessing = false;
  }
</script>

<div class="h-full overflow-y-auto">
  <div class="mx-auto flex max-w-275 flex-col gap-4 px-4 pt-5 pb-28 sm:px-6">
    <header class="flex flex-wrap items-end justify-between gap-x-6 gap-y-3">
      <div class="min-w-0">
        <h1 class="m-0 font-display text-[26px] leading-tight font-semibold">{tab.name}</h1>
        {#if tab.description}
          <p class="m-0 mt-1 text-[13px] text-foreground-muted">{tab.description}</p>
        {/if}
      </div>
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
    </header>

    <div class="flex flex-wrap items-center gap-2">
      <label
        class="flex h-8 min-w-48 flex-1 items-center gap-2 rounded-md border border-border bg-secondary px-2.5 focus-within:border-accent sm:max-w-80"
      >
        <Icon icon="mdi:magnify" width="16" class="shrink-0 text-foreground-muted" />
        <input
          type="text"
          placeholder="Filter this category"
          bind:value={searchQuery}
          class="min-w-0 flex-1 border-0 bg-transparent text-[13px] outline-none placeholder:text-foreground-subtle"
        />
        {#if searchQuery}
          <button
            type="button"
            class="flex cursor-pointer rounded p-0.5 text-foreground-muted hover:bg-muted hover:text-foreground"
            onclick={() => (searchQuery = "")}
            aria-label="Clear filter"
          >
            <Icon icon="mdi:close" width="14" />
          </button>
        {/if}
      </label>

      <div class="flex flex-wrap gap-1" role="group" aria-label="Show">
        {#each filters as f (f.id)}
          <button
            type="button"
            class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md px-2.5 text-[13px] hover:bg-muted {activeFilter ===
            f.id
              ? 'bg-muted font-semibold'
              : ''} {f.id === 'attention'
              ? 'text-error'
              : activeFilter === f.id
                ? 'text-foreground'
                : 'text-foreground-muted hover:text-foreground'}"
            aria-pressed={activeFilter === f.id}
            onclick={() => (filter = f.id)}
          >
            {f.label}
            <span class="text-xs text-foreground-subtle tabular-nums">{f.count}</span>
          </button>
        {/each}
      </div>

      {#if tweaksWithSnapshots.length > 0}
        <button
          type="button"
          class="ml-auto inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isLoading || isBatchProcessing}
          onclick={() => (showRevertAllDialog = true)}
        >
          <Icon icon="mdi:history" width="16" />
          Restore all
          <span class="text-xs text-foreground-subtle tabular-nums">{tweaksWithSnapshots.length}</span>
        </button>
      {/if}
    </div>

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

    {#if tweaksLoading && categoryTweaks.length === 0}
      <SkeletonCard />
    {:else if filteredTweaks.length === 0 && filteredApps.length === 0}
      <EmptyState
        icon={query ? "mdi:file-search-outline" : "mdi:package-variant"}
        title={query || activeFilter !== "all" ? "Nothing matches" : "No tweaks available"}
        description={query
          ? `Nothing in ${tab.name} matches "${searchQuery}"`
          : activeFilter !== "all"
            ? "No tweaks in this category match the selected filter"
            : "This category has no tweaks for your system"}
        actionText={query || activeFilter !== "all" ? "Show all" : undefined}
        onaction={() => {
          searchQuery = "";
          filter = "all";
        }}
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
        <section aria-labelledby="apps-heading-{tab.id}" class="mt-4">
          <h2 id="apps-heading-{tab.id}" class="m-0 mb-2 flex items-baseline gap-2 text-base font-semibold">
            Apps
            <span class="text-xs font-normal text-foreground-muted">{installedAppCount} installed</span>
          </h2>
          <div class="flex flex-col gap-2">
            {#each filteredApps as app (app.id)}
              <AppCard {app} />
            {/each}
          </div>
        </section>
      {/if}
    {/if}
  </div>
</div>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore Snapshots"
  message="Restore {tweaksWithSnapshots.length} tweak{tweaksWithSnapshots.length === 1
    ? ''
    : 's'} to their original state from saved snapshots?"
  confirmText="Restore Snapshots"
  variant="danger"
  onconfirm={handleRestoreSnapshots}
  oncancel={() => (showRevertAllDialog = false)}
/>
