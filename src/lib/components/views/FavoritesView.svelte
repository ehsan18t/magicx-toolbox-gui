<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { GroupedTweakList } from "$lib/components/tweaks";
  import { EmptyState, SkeletonCard } from "$lib/components/ui";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { batchRevertTweaks, loadingStateStore, loadingStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { canRestore, restoreMessage } from "$lib/utils/tweakPresentation";

  const favoriteTweaks = $derived(tweaksStore.list.filter((t) => favoritesStore.ids.includes(t.definition.id)));
  const filteredTweaks = $derived(favoriteTweaks.filter((t) => pageFilterStore.passes(t.definition.id)));
  const restorable = $derived(favoriteTweaks.filter(canRestore));
  const appliedCount = $derived(favoriteTweaks.filter((t) => t.status.is_applied).length);

  async function restoreAll() {
    const ids = restorable.map((t) => t.definition.id);
    const ok = await confirm({
      title: "Restore favorites?",
      message: restoreMessage(ids.length),
      confirmText: "Restore",
      variant: "danger",
    });
    if (ok) await batchRevertTweaks(ids);
  }

  async function clearAll() {
    const n = favoriteTweaks.length;
    const ok = await confirm({
      title: "Clear all favorites?",
      message: `Remove ${n === 1 ? "1 tweak" : `${n} tweaks`} from your favorites? This won't change the tweaks themselves.`,
      confirmText: "Clear favorites",
      variant: "danger",
    });
    if (!ok) return;
    favoritesStore.clear();
    toastStore.success("All favorites cleared");
  }
</script>

<PageLayout title="Favorites" description="Quick access to the tweaks you starred.">
  {#snippet aside()}
    {#if favoriteTweaks.length > 0}
      <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
        <p class="m-0 text-xs text-foreground-muted">
          <span class="font-semibold text-foreground tabular-nums">{favoriteTweaks.length}</span> starred ·
          <span class="font-semibold text-foreground tabular-nums">{appliedCount}</span> applied
        </p>
        {#if restorable.length > 0}
          <button
            type="button"
            class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            disabled={loadingStore.busy}
            onclick={restoreAll}
          >
            <Icon icon="mdi:history" width="16" />
            Restore all
            <span class="text-xs text-foreground-subtle tabular-nums">{restorable.length}</span>
          </button>
        {/if}
        <button
          type="button"
          class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md px-3 text-[13px] font-medium text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          disabled={loadingStore.busy}
          onclick={clearAll}
        >
          <Icon icon="mdi:star-off" width="16" />
          Clear favorites
        </button>
      </div>
    {/if}
  {/snippet}

  {#if loadingStateStore.tweaksLoading && favoriteTweaks.length === 0}
    <SkeletonCard />
  {:else if favoriteTweaks.length === 0}
    <EmptyState
      icon="mdi:star-outline"
      title="No favorites yet"
      description="Select the star on any tweak to keep it here for quick access."
      actionText="Browse tweaks"
      onaction={() => navigationStore.navigateToOverview()}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <EmptyState
      icon="mdi:file-search-outline"
      title="Nothing matches"
      description={`No favorites match "${pageFilterStore.query.trim()}"`}
      actionText="Search everywhere"
      onaction={() => pageFilterStore.searchEverywhere()}
    />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>
