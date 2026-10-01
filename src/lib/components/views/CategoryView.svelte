<script lang="ts">
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { AppCard, TweakCard } from "$lib/components/tweaks";
  import { ActionButton, EmptyState, SkeletonCard } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import type { TabDefinition } from "$lib/stores/navigation.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import {
    applyPendingChanges,
    batchRevertTweaks,
    loadingStateStore,
    loadingStore,
    pendingChangesStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";

  interface Props {
    tab: TabDefinition;
  }

  let { tab }: Props = $props();

  let searchQuery = $state("");
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  // Check if tweaks are still loading
  const tweaksLoading = $derived(loadingStateStore.tweaksLoading);

  // Get tweaks for this category
  const categoryTweaks = $derived(tweaksStore.list.filter((t) => t.definition.category_id === tab.id));

  // Filter tweaks by search
  const filteredTweaks = $derived.by(() => {
    if (!searchQuery.trim()) return categoryTweaks;
    const query = searchQuery.toLowerCase();
    return categoryTweaks.filter(
      (t) => t.definition.name.toLowerCase().includes(query) || t.definition.description.toLowerCase().includes(query),
    );
  });

  const categoryApps = $derived(appsStore.byCategory[tab.id] ?? []);
  const filteredApps = $derived.by(() => {
    if (!searchQuery.trim()) return categoryApps;
    const query = searchQuery.toLowerCase();
    return categoryApps.filter(
      (a) => a.name.toLowerCase().includes(query) || a.description.toLowerCase().includes(query),
    );
  });
  const installedAppCount = $derived(
    categoryApps.filter((a) => appsStore.status(a.id)?.presence.state === "installed").length,
  );

  // Stats
  const appliedCount = $derived(categoryTweaks.filter((t) => t.status.is_applied).length);
  const totalCount = $derived(categoryTweaks.length);

  // Tweaks with snapshots (can be restored)
  const tweaksWithSnapshots = $derived(categoryTweaks.filter((t) => t.status.has_backup));
  const snapshotCount = $derived(tweaksWithSnapshots.length);

  // Pending changes for this category
  const categoryPendingCount = $derived.by(() => {
    let count = 0;
    const pending = pendingChangesStore.all;
    const tweaks = tweaksStore.list;
    for (const [tweakId] of pending) {
      const tweak = tweaks.find((t) => t.definition.id === tweakId);
      if (tweak?.definition.category_id === tab.id) {
        count++;
      }
    }
    return count;
  });

  // Loading state
  const isLoading = $derived(categoryTweaks.some((t) => loadingStore.isLoading(t.definition.id)));

  async function handleApplyChanges() {
    showApplyAllDialog = false;
    isBatchProcessing = true;
    await applyPendingChanges();
    isBatchProcessing = false;
  }

  async function handleRestoreSnapshots() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    await batchRevertTweaks(tweaksWithSnapshots.map((t) => t.definition.id));

    isBatchProcessing = false;
  }

  function handleRestoreClick() {
    if (snapshotCount === 0) {
      toastStore.info("No snapshots available to restore in this category");
      return;
    }
    showRevertAllDialog = true;
  }

  function handleDiscardChanges() {
    pendingChangesStore.clearCategory(tab.id, tweaksStore.list);
  }
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden p-3">
  <!-- Toolbar -->
  <div class="flex flex-wrap items-center gap-3">
    <div
      class="flex max-w-full min-w-60 flex-1 items-center gap-2.5 rounded-lg border border-border bg-surface px-4 py-2.5 transition-all duration-200 focus-within:border-accent focus-within:bg-card"
    >
      <Icon icon="mdi:magnify" width="20" class="shrink-0 text-foreground-muted" />
      <input
        type="text"
        placeholder="Search tweaks and apps..."
        bind:value={searchQuery}
        class="flex-1 border-0 bg-transparent text-sm text-foreground outline-none placeholder:text-foreground-subtle"
      />
      {#if searchQuery}
        <button
          type="button"
          class="hover:bg-muted flex cursor-pointer items-center justify-center rounded border-0 bg-transparent p-1 text-foreground-muted transition-all duration-150 hover:text-foreground"
          onclick={() => (searchQuery = "")}
          aria-label="Clear search"
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      {/if}
    </div>
    <div class="flex w-full items-center justify-between">
      <div class="flex gap-2.5">
        <ActionButton
          intent="apply"
          icon="mdi:check-all"
          active={categoryPendingCount > 0}
          loading={isBatchProcessing}
          badgeCount={categoryPendingCount}
          badgeVariant="warning"
          onclick={() => (showApplyAllDialog = true)}
          disabled={categoryPendingCount === 0 || isLoading || isBatchProcessing}
        >
          Apply Changes
        </ActionButton>
        <ActionButton
          intent="discard"
          icon="mdi:close-circle-outline"
          onclick={handleDiscardChanges}
          disabled={categoryPendingCount === 0 || isLoading || isBatchProcessing}
          tooltip="Discard all pending changes in this category"
        >
          Discard
        </ActionButton>
        <ActionButton
          intent="restore"
          icon="mdi:restore"
          badgeCount={snapshotCount}
          badgeVariant="error"
          onclick={handleRestoreClick}
          disabled={snapshotCount === 0 || isLoading || isBatchProcessing}
          tooltip={snapshotCount === 0
            ? "No snapshots available"
            : `Restore ${snapshotCount} snapshot${snapshotCount > 1 ? "s" : ""}`}
        >
          Restore Snapshots
        </ActionButton>
      </div>

      <div class="flex flex-col items-center justify-center gap-0.5">
        <span class="text-xs font-bold text-foreground">{appliedCount} / {totalCount}</span>
        <span class="text-xs text-foreground-muted">Applied</span>
      </div>
    </div>
  </div>

  <!-- Tweaks Grid -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if tweaksLoading && categoryTweaks.length === 0}
      <SkeletonCard />
    {:else if filteredTweaks.length === 0 && filteredApps.length === 0}
      {#if searchQuery}
        <EmptyState
          icon="mdi:file-search-outline"
          title="No results found"
          description={`Nothing matches "${searchQuery}"`}
          actionText="Clear search"
          onaction={() => (searchQuery = "")}
        />
      {:else}
        <EmptyState
          icon="mdi:package-variant"
          title="No tweaks available"
          description="This category has no tweaks for your system"
        />
      {/if}
    {:else if filteredTweaks.length > 0}
      <div class="flex flex-col gap-3 pb-4 lg:grid lg:grid-cols-2 lg:gap-4">
        {#each filteredTweaks as tweak (tweak.definition.id)}
          <TweakCard {tweak} />
        {/each}
      </div>
    {/if}

    {#if filteredApps.length > 0}
      <section aria-labelledby="apps-heading-{tab.id}" class="pb-4">
        <h2 id="apps-heading-{tab.id}" class="m-0 mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
          <Icon icon="mdi:package-variant" width="18" class="text-foreground-muted" />
          Apps
          <span class="font-normal text-foreground-muted">· {installedAppCount} installed</span>
        </h2>
        <div class="flex flex-col gap-3 lg:grid lg:grid-cols-2 lg:gap-4">
          {#each filteredApps as app (app.id)}
            <AppCard {app} />
          {/each}
        </div>
      </section>
    {/if}
  </div>
</div>

<!-- Dialogs -->
<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply Pending Changes"
  message="Apply {categoryPendingCount} pending change(s)? Some tweaks may require a system restart."
  confirmText="Apply Changes"
  variant="default"
  onconfirm={handleApplyChanges}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Restore Snapshots"
  message="Restore {snapshotCount} tweak{snapshotCount > 1 ? 's' : ''} to their original state from saved snapshots?"
  confirmText="Restore Snapshots"
  variant="danger"
  onconfirm={handleRestoreSnapshots}
  oncancel={() => (showRevertAllDialog = false)}
/>
