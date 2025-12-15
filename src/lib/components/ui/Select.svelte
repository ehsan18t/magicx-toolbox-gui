<script lang="ts">
  import { cn } from "@/utils";
  import { cubicOut } from "svelte/easing";
  import { scale } from "svelte/transition";
  import Icon from "../Icon.svelte";

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
    class?: string;
    onchange?: (value: string | number) => void;
  }

  // eslint-disable-next-line prefer-const -- value must stay mutable for binding
  let { value = $bindable(), ...rest }: Props = $props();
  const {
    options,
    placeholder = "Select...",
    pending = false,
    loading = false,
    disabled = false,
    class: className = "",
    onchange,
  } = rest;

  let isOpen = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let menuEl = $state<HTMLDivElement | null>(null);
  let highlightedIndex = $state(-1);
  let menuPosition = $state({ top: 0, left: 0, width: 0 });

  const selectedOption = $derived(options.find((o) => o.value === value));
  const displayLabel = $derived(selectedOption?.label ?? placeholder);
  const isPlaceholder = $derived(!selectedOption);

  function updatePosition() {
    if (!triggerEl) return;
    const rect = triggerEl.getBoundingClientRect();
    menuPosition = {
      top: rect.bottom + 4,
      left: rect.left,
      width: rect.width,
    };
  }

  function open() {
    if (disabled || loading) return;
    updatePosition();
    isOpen = true;
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
    value = opt.value;
    onchange?.(opt.value);
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
</script>

<svelte:window onclick={handleClickOutside} onscroll={() => isOpen && updatePosition()} />

<div class={cn("relative", className)}>
  <button
    bind:this={triggerEl}
    type="button"
    onclick={toggle}
    onkeydown={handleKeydown}
    disabled={disabled || loading}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    class={cn(
      "flex h-10 w-full cursor-pointer items-center justify-between gap-2 rounded-lg border bg-surface px-3 text-sm transition-all duration-150",
      "border-border text-foreground",
      "hover:border-border-hover",
      "focus:border-accent focus:ring-2 focus:ring-accent/20 focus:outline-none",
      isOpen && "border-accent ring-2 ring-accent/20",
      pending && "border-warning/60 bg-warning/5 text-warning",
      loading && "cursor-wait opacity-70",
      disabled && "cursor-not-allowed opacity-60",
    )}
  >
    <span class={cn("truncate", isPlaceholder && "text-foreground-muted")}>
      {displayLabel}
    </span>
    <div class="flex shrink-0 items-center gap-1">
      {#if loading}
        <svg class="animate-spin h-4 w-4 text-foreground-muted" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
      {:else}
        <Icon
          icon="mdi:chevron-down"
          class={cn("h-4 w-4 text-foreground-muted transition-transform duration-150", isOpen && "rotate-180")}
        />
      {/if}
    </div>
  </button>
</div>

<!-- Dropdown rendered with fixed position to escape overflow:hidden containers -->
{#if isOpen}
  <div
    bind:this={menuEl}
    role="listbox"
    transition:scale={{ duration: 120, start: 0.95, opacity: 0, easing: cubicOut }}
    class="fixed z-9999 max-h-60 overflow-auto rounded-lg border border-border bg-elevated p-1 shadow-lg"
    style="top: {menuPosition.top}px; left: {menuPosition.left}px; width: {menuPosition.width}px;"
  >
    {#each options as opt, i (opt.value)}
      <button
        type="button"
        role="option"
        aria-selected={opt.value === value}
        disabled={opt.disabled}
        onclick={() => selectOption(opt)}
        onmouseenter={() => (highlightedIndex = i)}
        class={cn(
          "flex w-full cursor-pointer items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors duration-100",
          "text-foreground",
          highlightedIndex === i && "bg-accent/10",
          opt.value === value && "bg-accent/15 font-medium text-accent",
          opt.disabled && "cursor-not-allowed text-foreground-muted opacity-50",
        )}
      >
        <span class="truncate">{opt.label}</span>
        {#if opt.value === value}
          <Icon icon="mdi:check" class="h-4 w-4 shrink-0 text-accent" />
        {/if}
      </button>
    {/each}
  </div>
{/if}
