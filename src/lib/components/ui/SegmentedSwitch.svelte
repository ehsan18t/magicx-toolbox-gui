<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { cn } from "@/utils";

  export interface SegmentOption {
    value: number;
    label: string;
    /** Iconify name, e.g. 'mdi:check' */
    icon?: string;
    disabled?: boolean;
  }

  interface Props {
    value: number;
    options: SegmentOption[];
    pending?: boolean;
    loading?: boolean;
    disabled?: boolean;
    iconOnly?: boolean;
    /** Segments share the full available width. */
    stretch?: boolean;
    size?: "sm" | "md";
    /** Accessible name for the group. */
    label?: string;
    class?: string;
    onchange?: (value: number) => void;
  }

  let {
    value,
    options,
    pending = false,
    loading = false,
    disabled = false,
    iconOnly = false,
    stretch = false,
    size = "sm",
    label,
    class: className = "",
    onchange,
  }: Props = $props();

  const sizeClasses = {
    sm: { segment: "h-7 px-3 text-[13px]", segmentIconOnly: "h-7 px-2", icon: 14 },
    md: { segment: "h-8 px-3.5 text-sm", segmentIconOnly: "h-8 px-2.5", icon: 16 },
  };

  const currentSize = $derived(sizeClasses[size]);

  const selectedIndex = $derived(options.findIndex((o) => o.value === value));
  // With nothing selected the group still needs one tab stop.
  const tabStopIndex = $derived(selectedIndex >= 0 ? selectedIndex : options.findIndex((o) => !o.disabled));

  function handleClick(optValue: number) {
    if (disabled || loading || optValue === value) return;
    if (options.find((o) => o.value === optValue)?.disabled) return;
    onchange?.(optValue);
  }

  function nextSelectable(from: number, step: number): number | null {
    for (let i = 1; i <= options.length; i++) {
      const idx = (from + step * i + options.length * options.length) % options.length;
      if (!options[idx].disabled) return idx;
    }
    return null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (disabled || loading) return;

    let newIndex: number | null;
    if (e.key === "ArrowRight" || e.key === "ArrowDown") newIndex = nextSelectable(selectedIndex, 1);
    else if (e.key === "ArrowLeft" || e.key === "ArrowUp") newIndex = nextSelectable(selectedIndex, -1);
    else if (e.key === "Home") newIndex = nextSelectable(-1, 1);
    else if (e.key === "End") newIndex = nextSelectable(options.length, -1);
    else return;

    e.preventDefault();
    if (newIndex === null || newIndex === selectedIndex) return;
    onchange?.(options[newIndex].value);
    // Roving tab stop: focus follows the selection, or it stays on a now-untabbable segment.
    (e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>("[role='radio']")[newIndex]?.focus();
  }
</script>

<div
  role="radiogroup"
  aria-label={label}
  tabindex="-1"
  class={cn(
    "items-center gap-0.5 rounded-md border p-0.5 transition-colors duration-150",
    stretch ? "flex w-full" : "inline-flex max-w-full",
    pending ? "border-warning/50 bg-warning/10" : "border-border bg-secondary",
    disabled && "opacity-55",
    className,
  )}
  onkeydown={handleKeydown}
>
  {#each options as opt, i (opt.value)}
    {@const isSelected = opt.value === value}
    <button
      type="button"
      role="radio"
      aria-checked={isSelected}
      aria-label={iconOnly ? opt.label : undefined}
      tabindex={i === tabStopIndex ? 0 : -1}
      disabled={disabled || loading || opt.disabled}
      class={cn(
        "relative inline-flex min-w-0 items-center justify-center gap-1.5 rounded font-medium whitespace-nowrap",
        "outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed",
        stretch && "flex-1",
        iconOnly ? currentSize.segmentIconOnly : currentSize.segment,
        isSelected
          ? pending
            ? "bg-warning text-warning-foreground shadow-sm"
            : "bg-accent text-accent-foreground shadow-sm"
          : cn(
              "text-foreground-muted",
              opt.disabled && "opacity-40",
              !disabled && !loading && !opt.disabled && "cursor-pointer hover:bg-muted hover:text-foreground",
            ),
      )}
      onclick={() => handleClick(opt.value)}
      use:tooltip={opt.label}
    >
      {#if loading && isSelected}
        <Icon icon="mdi:loading" width={currentSize.icon} class="shrink-0 animate-spin" />
      {:else if opt.icon}
        <Icon icon={opt.icon} width={currentSize.icon} class="shrink-0" />
      {/if}
      {#if !iconOnly}
        <span class="truncate">{opt.label}</span>
      {/if}
    </button>
  {/each}
</div>
