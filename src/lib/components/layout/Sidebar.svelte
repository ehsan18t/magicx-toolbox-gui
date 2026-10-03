<script lang="ts">
  import { overflowHints } from "$lib/actions/overflowHints";
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { navigationStore, type TabDefinition, type TabId } from "$lib/stores/navigation.svelte";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { updateStore } from "$lib/stores/update.svelte";
  import { plural } from "$lib/utils/format";
  import { fade, reducedMotion } from "$lib/utils/motion";

  interface NavItem {
    label: string;
    icon: IconName;
    active: boolean;
    onclick: () => void;
    trailing?: string;
    trailingTone?: string;
    alert?: string;
    pending?: string;
  }

  const SKELETON_ROWS = 6;
  const SCROLL_PAGE_FRACTION = 0.6;
  const RAIL_MARKER = "absolute -top-0.5 -right-1 ring-2 ring-background";

  const isOpen = $derived(sidebarStore.isOpen);
  let moreAbove = $state(false);
  let moreBelow = $state(false);
  let scrollEl = $state<HTMLElement | null>(null);
  const activeTab = $derived(navigationStore.activeTab);
  const categoryStats = $derived(categoriesStore.stats);

  // One shared indicator glides between pages; when items shift under it (the pane opening, categories
  // loading) it jumps with them instead.
  let indicator = $state<{ x: number; y: number; glide: boolean } | null>(null);
  let indicatorTab: TabId | null = null;
  $effect(() => {
    void [activeTab, isOpen, navigationStore.fixedTabs.length, navigationStore.categoryTabs.length];
    const item = scrollEl?.querySelector<HTMLElement>('[aria-current="page"]');
    const glide = indicatorTab !== activeTab;
    indicatorTab = activeTab;
    indicator = item ? { x: item.offsetLeft, y: item.offsetTop + item.offsetHeight / 2, glide } : null;
  });
  const fadeFrom = $derived(sidebarStore.isOverlay ? "from-elevated" : "from-background");
  // Markers mean "act here": attention, or changes staged but not applied. Nothing else gets one.
  const pendingByCategory = $derived.by(() => {
    const counts: Record<string, number> = {};
    for (const change of pendingChangesStore.all.values()) {
      const category = tweaksStore.tweak(change.tweakId)?.definition.categoryId;
      if (category) counts[category] = (counts[category] ?? 0) + 1;
    }
    return counts;
  });

  function go(tab: TabDefinition) {
    navigationStore.navigateToTab(tab.id);
    sidebarStore.closeOverlay();
  }

  function fixedCount(id: TabId): number {
    if (id === "favorites") return favoritesStore.count;
    if (id === "snapshots") return tweaksStore.withSnapshot.length;
    return 0;
  }

  const footerItems: { label: string; icon: IconName; open: () => void; dot: boolean; active: boolean }[] = $derived([
    {
      label: updateStore.isAvailable ? "Update available" : "Updates",
      icon: "mdi:update",
      open: () => modalStore.open("update"),
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
    {
      label: "About",
      icon: "mdi:information-outline",
      open: () => modalStore.open("about"),
      dot: false,
      active: false,
    },
  ]);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== "Escape" || e.defaultPrevented || !sidebarStore.isOverlay) return;
    // An open dialog takes Escape, whichever window listener runs first.
    if (document.querySelector('[aria-modal="true"]')) return;
    e.preventDefault();
    sidebarStore.closeOverlay();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#snippet dot(classes: string)}
  <span class={["h-2 w-2 rounded-full", classes]} aria-hidden="true"></span>
{/snippet}

{#snippet navItem({
  label,
  icon,
  active,
  onclick,
  trailing = "",
  trailingTone = "text-foreground-subtle",
  alert = "",
  pending = "",
}: NavItem)}
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
    <span class="relative flex w-5 shrink-0 justify-center">
      <Icon {icon} size="lg" class={active ? "text-accent" : "text-foreground-muted group-hover:text-foreground"} />
      {#if alert && !isOpen}
        {@render dot(`${RAIL_MARKER} bg-error`)}
      {:else if pending && !isOpen}
        {@render dot(`${RAIL_MARKER} bg-warning`)}
      {/if}
    </span>
    {#if isOpen}
      <span class="min-w-0 flex-1 truncate">{label}</span>
      {#if alert}
        <Icon icon="mdi:alert-circle" size="xs" class="shrink-0 text-error" />
      {/if}
      {#if pending}
        {@render dot("shrink-0 bg-warning")}
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
    class="fixed inset-x-0 top-titlebar bottom-0 z-scrim animate-fade-in cursor-default bg-black/20"
    aria-label="Close navigation"
    tabindex="-1"
    onclick={() => sidebarStore.closeOverlay()}
  ></button>
{/if}

<nav
  class="relative h-full shrink-0 transition-[width] duration-slow ease-out {sidebarStore.isDockedExpanded
    ? 'w-64'
    : 'w-rail'}"
  aria-label="Main"
>
  <div
    class="flex h-full flex-col {sidebarStore.isOverlay
      ? 'absolute inset-y-0 left-0 z-drawer w-72 rounded-r-lg border border-l-0 border-border bg-elevated shadow-flyout transition-[width] duration-slow ease-out'
      : 'w-full'}"
  >
    <div class="relative flex min-h-0 flex-1 flex-col">
      <div
        bind:this={scrollEl}
        class="relative flex min-h-0 flex-1 scrollbar-none flex-col gap-0.5 overflow-x-hidden overflow-y-auto px-1.5 pt-1 pb-2"
        use:overflowHints={(above, below) => {
          moreAbove = above;
          moreBelow = below;
        }}
      >
        {#if indicator}
          <span
            class="pointer-events-none absolute top-0 left-0 h-4 w-0.75 -translate-y-1/2 animate-fade-in rounded-full bg-accent {indicator.glide
              ? 'transition-transform duration-slow'
              : ''}"
            style:transform="translate({indicator.x}px, {indicator.y}px)"
            aria-hidden="true"
          ></span>
        {/if}
        {#each navigationStore.fixedTabs as tab (tab.id)}
          {@const count = fixedCount(tab.id)}
          {@render navItem({
            label: tab.name,
            icon: tab.icon,
            active: activeTab === tab.id,
            onclick: () => go(tab),
            trailing: count > 0 ? String(count) : "",
          })}
        {/each}

        <div class="mx-2 my-2 h-px shrink-0 bg-border"></div>
        {#if isOpen}
          <div class="shrink-0 px-3 pb-1 text-xs font-semibold text-foreground-muted">Categories</div>
        {/if}

        {#each navigationStore.categoryTabs as tab (tab.id)}
          {@const s = categoryStats[tab.id]}
          {@const complete = !!s && s.total > 0 && s.applied === s.total}
          {@render navItem({
            label: tab.name,
            icon: tab.icon,
            active: activeTab === tab.id,
            onclick: () => go(tab),
            trailing: s ? `${s.applied}/${s.total}` : "",
            trailingTone: complete ? "text-success" : undefined,
            alert: s?.attention ? `${plural(s.attention, "needs", "need")} attention` : "",
            pending: pendingByCategory[tab.id] ? `${pendingByCategory[tab.id]} staged, not applied` : "",
          })}
        {/each}

        {#if tweaksStore.isLoading}
          {#each { length: SKELETON_ROWS }, i (i)}
            <div class="flex h-9 shrink-0 items-center gap-3 px-3">
              <div class="h-5 w-5 shrink-0 animate-pulse rounded bg-muted"></div>
              {#if isOpen}<div class="h-3.5 flex-1 animate-pulse rounded bg-muted"></div>{/if}
            </div>
          {/each}
        {/if}
      </div>
      {#if moreAbove}
        <div
          transition:fade={{ speed: "fast" }}
          class="pointer-events-none absolute inset-x-0 top-0 h-8 bg-linear-to-b to-transparent {fadeFrom}"
          aria-hidden="true"
        ></div>
      {/if}
      {#if moreBelow}
        <div
          transition:fade={{ speed: "fast" }}
          class="pointer-events-none absolute inset-x-0 bottom-0 flex h-12 items-end justify-center bg-linear-to-t from-40% to-transparent pb-1 {fadeFrom}"
        >
          <button
            type="button"
            class="pointer-events-auto flex h-6 w-8 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
            aria-label="Scroll for more"
            use:tooltip={"More below"}
            onclick={() =>
              scrollEl?.scrollBy({
                top: scrollEl.clientHeight * SCROLL_PAGE_FRACTION,
                behavior: reducedMotion() ? "auto" : "smooth",
              })}
          >
            <Icon icon="mdi:chevron-down" size="lg" />
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
          <Icon icon={item.icon} size="lg" />
          {#if item.dot}
            {@render dot("absolute top-1.5 right-1/2 translate-x-3 animate-pop-in bg-success ring-2 ring-background")}
          {/if}
        </button>
      {/each}
    </div>
  </div>
</nav>
