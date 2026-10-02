<script lang="ts">
  import { overflowHints } from "$lib/actions/overflowHints";
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { sidebarStore } from "$lib/stores/layout.svelte";
  import { openAboutModal, openUpdateModal } from "$lib/stores/modal.svelte";
  import { navigationStore, type TabDefinition } from "$lib/stores/navigation.svelte";
  import { categoriesStore, getCategoryStats, pendingChangesStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { updateStore } from "$lib/stores/update.svelte";

  const isOpen = $derived(sidebarStore.isOpen);
  let moreAbove = $state(false);
  let moreBelow = $state(false);
  let scrollEl = $state<HTMLElement | null>(null);
  const activeTab = $derived(navigationStore.activeTab);
  const categoryStats = $derived(getCategoryStats());
  const snapshotCount = $derived(tweaksStore.list.filter((t) => t.status.has_backup).length);
  // Markers mean "act here": attention, or changes staged but not applied. Nothing else gets one.
  const pendingByCategory = $derived.by(() => {
    const counts: Record<string, number> = {};
    for (const change of pendingChangesStore.all.values()) {
      const category = tweaksStore.getById(change.tweakId)?.definition.category_id;
      if (category) counts[category] = (counts[category] ?? 0) + 1;
    }
    return counts;
  });

  function go(tab: TabDefinition) {
    navigationStore.navigateToTab(tab.id);
    sidebarStore.closeOverlay();
  }

  function fixedCount(id: string): number {
    if (id === "favorites") return favoritesStore.count;
    if (id === "snapshots") return snapshotCount;
    return 0;
  }

  const footerItems = $derived([
    {
      label: updateStore.isAvailable ? "Update available" : "Updates",
      icon: "mdi:update",
      open: openUpdateModal,
      dot: updateStore.isAvailable,
      active: false,
    },
    {
      label: "Settings",
      icon: "mdi:cog-outline",
      open: () => navigationStore.navigateToTab("settings"),
      dot: false,
      active: activeTab === "settings",
    },
    { label: "About", icon: "mdi:information-outline", open: openAboutModal, dot: false, active: false },
  ]);

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
  alert = "",
  pending = "",
)}
  <button
    type="button"
    class="group relative flex h-9 w-full shrink-0 cursor-pointer items-center gap-3 rounded-md px-3 text-left text-sm text-foreground {active
      ? 'bg-muted'
      : 'hover:bg-muted'}"
    aria-current={active ? "page" : undefined}
    aria-label={[label, trailing, alert, pending].filter(Boolean).join(", ")}
    use:tooltip={isOpen
      ? [alert, pending].filter(Boolean).join(" · ") || null
      : [label, trailing, alert, pending].filter(Boolean).join(" · ")}
    {onclick}
  >
    {#if active}
      <span class="absolute top-1/2 left-0 h-4 w-0.75 -translate-y-1/2 rounded-full bg-accent"></span>
    {/if}
    <span class="relative flex w-5 shrink-0 justify-center">
      <Icon {icon} width="18" class={active ? "text-accent" : "text-foreground-muted group-hover:text-foreground"} />
      {#if alert && !isOpen}
        <span class="absolute -top-0.5 -right-1 h-2 w-2 rounded-full bg-error ring-2 ring-background"></span>
      {:else if pending && !isOpen}
        <span class="absolute -top-0.5 -right-1 h-2 w-2 rounded-full bg-warning ring-2 ring-background"></span>
      {/if}
    </span>
    {#if isOpen}
      <span class="min-w-0 flex-1 truncate">{label}</span>
      {#if alert}
        <Icon icon="mdi:alert-circle" width="14" class="shrink-0 text-error" />
      {/if}
      {#if pending}
        <span class="h-2 w-2 shrink-0 rounded-full bg-warning" aria-hidden="true"></span>
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
    class="fixed inset-x-0 top-12 bottom-0 z-scrim cursor-default bg-black/20"
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
      ? 'absolute inset-y-0 left-0 z-drawer w-72 animate-rise-in rounded-r-lg border border-l-0 border-border bg-elevated shadow-flyout'
      : 'w-full'}"
  >
    <div class="relative flex min-h-0 flex-1 flex-col">
      <div
        bind:this={scrollEl}
        class="nav-scroll flex min-h-0 flex-1 flex-col gap-0.5 overflow-x-hidden overflow-y-auto px-1.5 pt-1 pb-2"
        use:overflowHints={(above, below) => {
          moreAbove = above;
          moreBelow = below;
        }}
      >
        {#each navigationStore.fixedTabs as tab (tab.id)}
          {@const count = fixedCount(tab.id)}
          {@render navItem(
            tab.name,
            tab.icon || "mdi:folder",
            activeTab === tab.id,
            () => go(tab),
            count > 0 ? String(count) : "",
            "text-foreground-subtle",
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
            s?.attention ? `${s.attention} need${s.attention === 1 ? "s" : ""} attention` : "",
            pendingByCategory[tab.id] ? `${pendingByCategory[tab.id]} staged, not applied` : "",
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
      {#if moreAbove}
        <div
          class="pointer-events-none absolute inset-x-0 top-0 h-8 bg-linear-to-b to-transparent {sidebarStore.isOverlay
            ? 'from-elevated'
            : 'from-background'}"
          aria-hidden="true"
        ></div>
      {/if}
      {#if moreBelow}
        <div
          class="pointer-events-none absolute inset-x-0 bottom-0 flex h-12 items-end justify-center bg-linear-to-t from-40% to-transparent pb-1 {sidebarStore.isOverlay
            ? 'from-elevated'
            : 'from-background'}"
        >
          <button
            type="button"
            class="pointer-events-auto flex h-6 w-8 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
            aria-label="Scroll for more"
            use:tooltip={"More below"}
            onclick={() =>
              scrollEl?.scrollBy({
                top: scrollEl.clientHeight * 0.6,
                behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth",
              })}
          >
            <Icon icon="mdi:chevron-down" width="18" />
          </button>
        </div>
      {/if}
    </div>

    <div
      class="flex shrink-0 gap-0.5 border-t border-border px-1.5 py-1.5 {isOpen
        ? 'flex-row justify-around'
        : 'flex-col'}"
    >
      {#each footerItems as item (item.label)}
        <button
          type="button"
          class="relative flex h-9 shrink-0 cursor-pointer items-center justify-center rounded-md hover:bg-muted hover:text-foreground {isOpen
            ? 'flex-1'
            : 'w-full'} {item.active ? 'bg-muted text-accent' : 'text-foreground-muted'}"
          aria-label={item.label}
          aria-current={item.active ? "page" : undefined}
          use:tooltip={item.label}
          onclick={() => {
            sidebarStore.closeOverlay();
            item.open();
          }}
        >
          <Icon icon={item.icon} width="18" />
          {#if item.dot}
            <span
              class="absolute top-1.5 right-1/2 h-2 w-2 translate-x-3 rounded-full bg-success ring-2 ring-background"
            ></span>
          {/if}
        </button>
      {/each}
    </div>
  </div>
</nav>

<style>
  .nav-scroll {
    scrollbar-width: none;
  }
</style>
