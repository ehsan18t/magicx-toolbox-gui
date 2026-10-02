<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { sidebarStore } from "$lib/stores/layout.svelte";
  import { openAboutModal, openSettingsModal, openUpdateModal } from "$lib/stores/modal.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { categoriesStore, getCategoryStats, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { updateStore } from "$lib/stores/update.svelte";

  const isOpen = $derived(sidebarStore.isOpen);
  const activeTab = $derived(navigationStore.activeTab);
  const categoryStats = $derived(getCategoryStats());
  const snapshotCount = $derived(tweaksStore.list.filter((t) => t.status.has_backup).length);

  function go(tab: TabDefinition) {
    navigationStore.navigateToTab(tab.id);
    sidebarStore.closeOverlay();
  }

  function fixedCount(id: string): number {
    if (id === "favorites") return favoritesStore.count;
    if (id === "snapshots") return snapshotCount;
    return 0;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== "Escape" || !sidebarStore.isOverlay) return;
    e.preventDefault();
    sidebarStore.closeOverlay();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#snippet navItem(
  label: string,
  icon: string,
  active: boolean,
  onclick: () => void,
  trailing: string,
  trailingTone: string,
  dot: string | null,
  alert = "",
)}
  <button
    type="button"
    class="group relative flex h-9 w-full shrink-0 cursor-pointer items-center gap-3 rounded-md px-3 text-left text-sm text-foreground {active
      ? 'bg-muted'
      : 'hover:bg-muted'}"
    aria-current={active ? "page" : undefined}
    aria-label={[label, trailing, alert].filter(Boolean).join(", ")}
    use:tooltip={isOpen ? (alert ? `${alert}` : null) : [label, trailing, alert].filter(Boolean).join(" · ")}
    {onclick}
  >
    {#if active}
      <span class="absolute top-1/2 left-0 h-4 w-0.75 -translate-y-1/2 rounded-full bg-accent"></span>
    {/if}
    <span class="relative flex w-5 shrink-0 justify-center">
      <Icon {icon} width="18" class={active ? "text-accent" : "text-foreground-muted group-hover:text-foreground"} />
      {#if alert && !isOpen}
        <span class="absolute -top-0.5 -right-1 h-2 w-2 rounded-full bg-error ring-2 ring-background"></span>
      {:else if dot && !isOpen}
        <span class="absolute -top-0.5 -right-1 h-2 w-2 rounded-full ring-2 ring-background {dot}"></span>
      {/if}
    </span>
    {#if isOpen}
      <span class="min-w-0 flex-1 truncate">{label}</span>
      {#if alert}
        <Icon icon="mdi:alert-circle" width="14" class="shrink-0 text-error" />
      {/if}
      {#if trailing}
        <span class="shrink-0 text-xs tabular-nums {trailingTone}">{trailing}</span>
      {/if}
    {/if}
  </button>
{/snippet}

{#if sidebarStore.isOverlay}
  <button
    type="button"
    class="fixed inset-x-0 top-12 bottom-0 z-40 cursor-default bg-black/20"
    aria-label="Close navigation"
    tabindex="-1"
    onclick={() => sidebarStore.closeOverlay()}
  ></button>
{/if}

<nav
  class="relative h-full shrink-0 transition-[width] duration-200 ease-out {sidebarStore.isDockedExpanded
    ? 'w-64'
    : 'w-14'}"
  aria-label="Main"
>
  <div
    class="flex h-full flex-col {sidebarStore.isOverlay
      ? 'absolute inset-y-0 left-0 z-50 w-72 animate-slide-in-up rounded-r-lg border border-l-0 border-border bg-elevated shadow-flyout'
      : 'w-full'}"
  >
    <div class="nav-scroll flex min-h-0 flex-1 flex-col gap-0.5 overflow-x-hidden overflow-y-auto px-1.5 pt-1 pb-2">
      {#each navigationStore.fixedTabs as tab (tab.id)}
        {@const count = fixedCount(tab.id)}
        {@render navItem(
          tab.name,
          tab.icon || "mdi:folder",
          activeTab === tab.id,
          () => go(tab),
          count > 0 ? String(count) : "",
          "text-foreground-subtle",
          count > 0 ? (tab.id === "favorites" ? "bg-warning" : "bg-accent") : null,
        )}
      {/each}

      <div class="mx-2 my-2 h-px shrink-0 bg-border"></div>
      {#if isOpen}
        <div class="shrink-0 px-3 pb-1 text-xs font-semibold text-foreground-muted">Categories</div>
      {/if}

      {#each navigationStore.categoryTabs as tab (tab.id)}
        {@const s = categoryStats[tab.id]}
        {@const complete = !!s && s.total > 0 && s.applied === s.total}
        {@render navItem(
          tab.name,
          tab.icon || "mdi:folder",
          activeTab === tab.id,
          () => go(tab),
          s ? `${s.applied}/${s.total}` : "",
          complete ? "text-success" : "text-foreground-subtle",
          s && s.applied > 0 ? "bg-accent" : null,
          s?.attention ? `${s.attention} need${s.attention === 1 ? "s" : ""} attention` : "",
        )}
      {/each}

      {#if categoriesStore.isLoading}
        {#each [0, 1, 2, 3, 4, 5] as i (i)}
          <div class="flex h-9 shrink-0 items-center gap-3 px-3">
            <div class="h-5 w-5 shrink-0 animate-pulse rounded bg-muted"></div>
            {#if isOpen}<div class="h-3.5 flex-1 animate-pulse rounded bg-muted"></div>{/if}
          </div>
        {/each}
      {/if}
    </div>

    <div class="flex shrink-0 flex-col gap-0.5 border-t border-border px-1.5 py-1.5">
      {@render navItem(
        updateStore.isAvailable ? "Update available" : "Updates",
        "mdi:update",
        false,
        () => {
          sidebarStore.closeOverlay();
          openUpdateModal();
        },
        "",
        "",
        updateStore.isAvailable ? "bg-success" : null,
      )}
      {@render navItem(
        "Settings",
        "mdi:cog-outline",
        false,
        () => {
          sidebarStore.closeOverlay();
          openSettingsModal();
        },
        "",
        "",
        null,
      )}
      {@render navItem(
        "About",
        "mdi:information-outline",
        false,
        () => {
          sidebarStore.closeOverlay();
          openAboutModal();
        },
        "",
        "",
        null,
      )}
    </div>
  </div>
</nav>

<style>
  .nav-scroll {
    scrollbar-width: none;
  }
</style>
