<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { pop } from "$lib/utils/motion";
  import { tick } from "svelte";
  import Spinner from "./Spinner.svelte";

  interface Option {
    value: string | number;
    label: string;
    disabled?: boolean;
  }

  interface Props {
    value: string | number | null;
    options: Option[];
    placeholder?: string;
    pending?: boolean;
    loading?: boolean;
    disabled?: boolean;
    /** Accessible name for the control. */
    label?: string;
    class?: string;
    onchange?: (value: string | number) => void;
  }

  let {
    value,
    options,
    placeholder = "Select...",
    pending = false,
    loading = false,
    disabled = false,
    label,
    class: className = "",
    onchange,
  }: Props = $props();

  const instanceId =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? `select-${crypto.randomUUID()}`
      : `select-${Math.random().toString(36).slice(2)}`;
  const listboxId = `${instanceId}-listbox`;
  // Index, not value: labels carry spaces and colons, which break the id reference.
  const optionId = (i: number) => `${instanceId}-option-${i}`;

  let isOpen = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let menuEl = $state<HTMLDivElement | null>(null);
  let highlightedIndex = $state(-1);
  let menuPosition = $state({ top: 0, left: 0, minWidth: 0, maxWidth: 0, above: false });
  const MENU_GAP = 4;
  const VIEWPORT_GUTTER = 8;

  const selectedOption = $derived(options.find((o) => o.value === value));
  const displayLabel = $derived(selectedOption?.label ?? placeholder);
  const isPlaceholder = $derived(!selectedOption);
  const highlightedOptionId = $derived(
    isOpen && highlightedIndex >= 0 && options[highlightedIndex] ? optionId(highlightedIndex) : undefined,
  );

  async function updatePosition() {
    if (!triggerEl) return;
    const rect = triggerEl.getBoundingClientRect();

    const maxWidth = window.innerWidth - VIEWPORT_GUTTER * 2;
    const below = rect.bottom + MENU_GAP;
    menuPosition = { top: below, left: rect.left, minWidth: Math.min(rect.width, maxWidth), maxWidth, above: false };

    await tick();
    if (!menuEl) return;
    // Offset size, not the bounding rect: the opening pop scales the menu.
    const height = menuEl.offsetHeight;
    const width = menuEl.offsetWidth;
    const above =
      below + height > window.innerHeight - VIEWPORT_GUTTER && rect.top - height - MENU_GAP > VIEWPORT_GUTTER;
    const top = above ? rect.top - height - MENU_GAP : below;
    const left = Math.max(VIEWPORT_GUTTER, Math.min(rect.left, window.innerWidth - VIEWPORT_GUTTER - width));
    menuPosition = { ...menuPosition, top, left, above };
  }

  async function open() {
    if (disabled || loading) return;
    isOpen = true;
    await updatePosition();
    const selectedIdx = options.findIndex((o) => o.value === value);
    highlightedIndex = selectedIdx >= 0 ? selectedIdx : options.findIndex((o) => !o.disabled);
  }

  function close() {
    isOpen = false;
    highlightedIndex = -1;
  }

  function toggle() {
    if (isOpen) close();
    else open();
  }

  function selectOption(opt: Option) {
    if (opt.disabled) return;
    // Controlled: the parent may decline the change, so the trigger shows only what it passes back.
    if (opt.value !== value) onchange?.(opt.value);
    close();
    triggerEl?.focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (disabled || loading) return;

    switch (e.key) {
      case "Enter":
      case " ":
        e.preventDefault();
        if (isOpen && highlightedIndex >= 0) {
          const opt = options[highlightedIndex];
          if (opt && !opt.disabled) selectOption(opt);
        } else {
          open();
        }
        break;
      case "ArrowDown":
        e.preventDefault();
        if (!isOpen) {
          open();
        } else {
          moveHighlight(1);
        }
        break;
      case "ArrowUp":
        e.preventDefault();
        if (!isOpen) {
          open();
        } else {
          moveHighlight(-1);
        }
        break;
      case "Escape":
        if (isOpen) {
          e.preventDefault();
          close();
          triggerEl?.focus();
        }
        break;
      case "Tab":
        if (isOpen) close();
        break;
    }
  }

  function moveHighlight(direction: number) {
    const len = options.length;
    let next = highlightedIndex;
    for (let i = 0; i < len; i++) {
      next = (next + direction + len) % len;
      if (!options[next]?.disabled) {
        highlightedIndex = next;
        break;
      }
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (!isOpen) return;
    const target = e.target as Node;
    if (!triggerEl?.contains(target) && !menuEl?.contains(target)) {
      close();
    }
  }

  // Keep the highlighted option in view while navigating.
  $effect(() => {
    if (!isOpen || !menuEl || highlightedIndex < 0) return;
    menuEl
      .querySelector<HTMLElement>(`#${CSS.escape(optionId(highlightedIndex))}`)
      ?.scrollIntoView({ block: "nearest" });
  });

  // Set up scroll listeners on scrollable ancestors
  $effect(() => {
    if (!triggerEl) return;

    let el: HTMLElement | null = triggerEl.parentElement;
    const scrollListeners: Array<{ el: Element; handler: EventListener }> = [];

    while (el) {
      const style = window.getComputedStyle(el);
      const isScrollable = /(auto|scroll)/.test(style.overflow + style.overflowY + style.overflowX);

      if (isScrollable) {
        const handler = () => {
          if (isOpen) close();
        };
        el.addEventListener("scroll", handler, { passive: true });
        scrollListeners.push({ el, handler });
      }

      el = el.parentElement;
    }

    return () => {
      scrollListeners.forEach(({ el, handler }) => {
        el.removeEventListener("scroll", handler);
      });
    };
  });
</script>

<svelte:window onclick={handleClickOutside} onscroll={() => isOpen && close()} />

<div class={cn("relative", className)}>
  <button
    bind:this={triggerEl}
    type="button"
    role="combobox"
    onclick={toggle}
    onkeydown={handleKeydown}
    disabled={disabled || loading}
    aria-label={label ? `${label}: ${displayLabel}` : undefined}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    aria-controls={listboxId}
    aria-activedescendant={highlightedOptionId}
    class={cn(
      "flex h-8 w-full cursor-pointer items-center justify-between gap-2 rounded-md border bg-secondary px-3 text-ui",
      "border-border text-foreground hover:border-border-hover hover:bg-secondary-hover",
      isOpen && "border-accent",
      pending && "border-warning/60 bg-warning/10 text-warning",
      loading && "cursor-wait opacity-70",
      disabled && "cursor-not-allowed opacity-60",
    )}
  >
    <span class={cn("truncate", isPlaceholder && "text-foreground-muted")}>
      {displayLabel}
    </span>
    <div class="flex shrink-0 items-center gap-1">
      {#if loading}
        <Spinner size="sm" class="text-foreground-muted" />
      {:else}
        <Icon
          icon="mdi:chevron-down"
          class={cn("h-4 w-4 text-foreground-muted transition-transform duration-normal", isOpen && "rotate-180")}
        />
      {/if}
    </div>
  </button>
</div>

<!-- Dropdown rendered with fixed position to escape overflow:hidden containers -->
{#if isOpen}
  <div
    bind:this={menuEl}
    id={listboxId}
    role="listbox"
    transition:pop
    class="fixed z-popover max-h-72 space-y-0.5 overflow-auto rounded-lg border border-border bg-elevated p-1 shadow-flyout {menuPosition.above
      ? 'origin-bottom'
      : 'origin-top'}"
    style="top: {menuPosition.top}px; left: {menuPosition.left}px; min-width: {menuPosition.minWidth}px; max-width: {menuPosition.maxWidth}px;"
  >
    {#each options as opt, i (opt.value)}
      <button
        type="button"
        role="option"
        id={optionId(i)}
        aria-selected={opt.value === value}
        aria-disabled={opt.disabled}
        disabled={opt.disabled}
        tabindex={-1}
        onclick={() => selectOption(opt)}
        onmouseenter={() => (highlightedIndex = i)}
        class={cn(
          "relative flex w-full cursor-pointer items-center rounded px-3 py-1.5 text-left text-ui text-foreground",
          highlightedIndex === i && "bg-muted",
          opt.value === value && "bg-muted font-medium",
          opt.disabled && "cursor-not-allowed text-foreground-muted opacity-50",
        )}
      >
        {#if opt.value === value}
          <span class="absolute top-1/2 left-0 h-4 w-0.75 -translate-y-1/2 rounded-full bg-accent"></span>
        {/if}
        <span class="min-w-0 wrap-break-word">{opt.label}</span>
      </button>
    {/each}
  </div>
{/if}
