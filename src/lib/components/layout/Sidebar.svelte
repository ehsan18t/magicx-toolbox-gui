<script lang="ts" module>
  const SKELETON_ROWS = 6;
  const SCROLL_PAGE_FRACTION = 0.6;
</script>

<script lang="ts">
  import { overflowHints } from "$lib/attachments/overflowHints";
  import { card, IconButton, indicator as indicatorBar, Skeleton } from "$lib/components/ui";
  import { navigationStore, pageTweaks, type TabId } from "$lib/stores/navigation.svelte";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { isComplete } from "$lib/utils/tweakPresentation";
  import { plural } from "$lib/utils/format";
  import { fade, reducedMotion } from "$lib/utils/motion";
  import SidebarFooter from "./SidebarFooter.svelte";
  import SidebarNavItem from "./SidebarNavItem.svelte";

  const isOpen = $derived(sidebarStore.isOpen);
  let moreAbove = $state(false);
  let moreBelow = $state(false);
  let scrollEl = $state<HTMLElement | null>(null);
  let drawerEl = $state<HTMLElement | null>(null);
  const activeTab = $derived(navigationStore.activeTab);
  const categoryStats = $derived(categoriesStore.stats);
  const pendingByCategory = $derived(pendingChangesStore.countByCategory);
  const fadeFrom = $derived(sidebarStore.isOverlay ? "from-elevated" : "from-background");

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

  // The drawer takes focus while open and hands it back on close, unless the reader moved it elsewhere.
  $effect(() => {
    if (!sidebarStore.isOverlay || !scrollEl) return;
    const nav = scrollEl;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    (nav.querySelector<HTMLElement>('[aria-current="page"]') ?? nav.querySelector<HTMLElement>("button"))?.focus();
    return () => {
      const focused = document.activeElement;
      if (opener?.isConnected && (focused === document.body || nav.contains(focused))) opener.focus();
    };
  });

  function go(navigate: () => void) {
    navigate();
    sidebarStore.closeOverlay();
  }

  // The drawer is modal: Tab wraps inside it instead of reaching the content under the scrim.
  function containTab(e: KeyboardEvent) {
    if (!drawerEl) return;
    const items = [...drawerEl.querySelectorAll<HTMLElement>("button:not([disabled]):not([tabindex='-1'])")];
    const first = items[0];
    const last = items.at(-1);
    if (!first || !last) return;
    const active = document.activeElement;
    const edge = e.shiftKey ? first : last;
    if (drawerEl.contains(active) && active !== edge) return;
    e.preventDefault();
    (e.shiftKey ? last : first).focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.defaultPrevented || !sidebarStore.isOverlay) return;
    // An open dialog takes Escape and Tab, whichever window listener runs first.
    if (document.querySelector('[aria-modal="true"]')) return;
    if (e.key === "Tab") containTab(e);
    else if (e.key === "Escape") {
      e.preventDefault();
      sidebarStore.closeOverlay();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if sidebarStore.isOverlay}
  <button
    type="button"
    class="fixed inset-x-0 top-titlebar bottom-0 z-scrim animate-fade-in cursor-default bg-scrim-light"
    aria-label="Close navigation"
    tabindex="-1"
    onclick={() => sidebarStore.closeOverlay()}
  ></button>
{/if}

<nav
  class="relative h-full shrink-0 transition-[width] duration-slow ease-out {sidebarStore.isDockedExpanded
    ? 'w-sidebar'
    : 'w-rail'}"
  aria-label="Main"
>
  <div
    bind:this={drawerEl}
    class="flex h-full flex-col {sidebarStore.isOverlay
      ? card({
          elevation: 'flyout',
          class:
            'absolute inset-y-0 left-0 z-drawer w-drawer rounded-l-none border-l-0 transition-[width] duration-slow ease-out',
        })
      : 'w-full'}"
  >
    <div class="relative flex min-h-0 flex-1 flex-col">
      <div
        bind:this={scrollEl}
        class="relative flex min-h-0 flex-1 scrollbar-none flex-col gap-0.5 overflow-x-hidden overflow-y-auto px-1.5 pt-1 pb-2"
        {@attach overflowHints((above, below) => {
          moreAbove = above;
          moreBelow = below;
        })}
      >
        {#if indicator}
          <span
            class={indicatorBar({
              class: [
                "pointer-events-none top-0 animate-fade-in",
                indicator.glide && "transition-transform duration-slow",
              ],
            })}
            style:transform="translate({indicator.x}px, {indicator.y}px)"
            aria-hidden="true"
          ></span>
        {/if}
        {#each navigationStore.fixedTabs as tab (tab.id)}
          {@const count = pageTweaks(tab.id)?.length ?? 0}
          <SidebarNavItem
            label={tab.name}
            icon={tab.icon}
            active={activeTab === tab.id}
            onclick={() => go(() => navigationStore.navigateToPage(tab.id))}
            trailing={count > 0 ? String(count) : ""}
          />
        {/each}

        <div class="mx-2 my-2 h-px shrink-0 bg-border"></div>
        {#if isOpen}
          <div class="shrink-0 px-3 pb-1 text-xs font-semibold text-foreground-muted">Categories</div>
        {/if}

        {#each navigationStore.categoryTabs as tab (tab.id)}
          {@const s = categoryStats[tab.id]}
          <SidebarNavItem
            label={tab.name}
            icon={tab.icon}
            active={activeTab === tab.id}
            onclick={() => go(() => navigationStore.navigateToCategory(tab.id))}
            trailing={s ? `${s.applied}/${s.total}` : ""}
            trailingTone={s && isComplete(s) ? "success" : undefined}
            alert={s?.attention ? `${plural(s.attention, "needs", "need")} attention` : ""}
            pending={pendingByCategory[tab.id] ? `${pendingByCategory[tab.id]} staged, not applied` : ""}
          />
        {/each}

        {#if tweaksStore.isLoading}
          {#each { length: SKELETON_ROWS }, i (i)}
            <div class="flex h-9 shrink-0 items-center gap-3 px-3">
              <Skeleton class="h-5 w-5 shrink-0" />
              {#if isOpen}<Skeleton class="h-3.5 flex-1" />{/if}
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
          <IconButton
            icon="mdi:chevron-down"
            size="sm"
            label="Scroll for more"
            tooltip="More below"
            class="pointer-events-auto"
            onclick={() =>
              scrollEl?.scrollBy({
                top: scrollEl.clientHeight * SCROLL_PAGE_FRACTION,
                behavior: reducedMotion() ? "auto" : "smooth",
              })}
          />
        </div>
      {/if}
    </div>

    <SidebarFooter />
  </div>
</nav>
