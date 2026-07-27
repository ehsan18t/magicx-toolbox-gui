<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { cn } from "@/utils";

  export interface SegmentOption {
    value: number;
    label: string;
    /** Optional icon to show (Iconify format, e.g., 'mdi:check') */
    icon?: string;
    /** Segment is shown but cannot be selected (e.g. System Default with no snapshot). */
    disabled?: boolean;
  }

  interface Props {
    /** Currently selected value */
    value: number;
    /** Available options */
    options: SegmentOption[];
    /** Show pending state styling */
    pending?: boolean;
    /** Show loading spinner on selected segment */
    loading?: boolean;
    /** Disable all interactions */
    disabled?: boolean;
    /** Show icons only (hide labels) */
    iconOnly?: boolean;
    /** Size variant */
    size?: "sm" | "md";
    /** Additional CSS classes */
    class?: string;
    /** Change handler */
    onchange?: (value: number) => void;
  }

  let {
    value,
    options,
    pending = false,
    loading = false,
    disabled = false,
    iconOnly = false,
    size = "sm",
    class: className = "",
    onchange,
  }: Props = $props();

  // Size-specific classes
  const sizeClasses = {
    sm: {
      track: "p-0.5 gap-0.5",
      segment: "px-2.5 py-1 text-xs",
      segmentIconOnly: "px-2 py-1",
      icon: 16,
    },
    md: {
      track: "p-1 gap-1",
      segment: "px-3.5 py-1.5 text-sm",
      segmentIconOnly: "px-2.5 py-1.5",
      icon: 18,
    },
  };

  const currentSize = $derived(sizeClasses[size]);

  // Find index of selected option for keyboard navigation
  const selectedIndex = $derived(options.findIndex((o) => o.value === value));

  function handleClick(optValue: number) {
    if (disabled || loading) return;
    if (options.find((o) => o.value === optValue)?.disabled) return;
    onchange?.(optValue);
  }

  /** Next selectable segment in `step` direction, skipping disabled ones. Null if there is none. */
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

    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      e.preventDefault();
      newIndex = nextSelectable(selectedIndex, 1);
    } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      e.preventDefault();
      newIndex = nextSelectable(selectedIndex, -1);
    } else if (e.key === "Home") {
      e.preventDefault();
      newIndex = nextSelectable(-1, 1);
    } else if (e.key === "End") {
      e.preventDefault();
      newIndex = nextSelectable(options.length, -1);
    } else {
      return;
    }

    if (newIndex !== null && newIndex !== selectedIndex) {
      onchange?.(options[newIndex].value);
    }
  }
</script>

<div
  role="radiogroup"
  tabindex="-1"
  class={cn(
    "inline-flex items-center rounded-full  transition-colors duration-200",
    pending ? "bg-warning/15" : "bg-accent-foreground/15 shadow-inner",
    disabled && "opacity-60",
    currentSize.track,
    className,
  )}
  onkeydown={handleKeydown}
>
  {#each options as opt (opt.value)}
    {@const isSelected = opt.value === value}
    <button
      type="button"
      role="radio"
      aria-checked={isSelected}
      tabindex={isSelected ? 0 : -1}
      disabled={disabled || loading || opt.disabled}
      class={cn(
        "relative inline-flex items-center justify-center gap-1.5 font-medium transition-all duration-150",
        "rounded-full outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
        "disabled:cursor-not-allowed",
        currentSize.segment,
        iconOnly && currentSize.segmentIconOnly,
        isSelected
          ? pending
            ? "text-warning-foreground scale-[1.02] bg-warning shadow-md"
            : "scale-[1.02] bg-accent text-accent-foreground shadow-md"
          : cn(
              "text-foreground-muted",
              opt.disabled && "opacity-40",
              !disabled && !loading && !opt.disabled && "cursor-pointer hover:bg-white/5 hover:text-foreground",
            ),
      )}
      onclick={() => handleClick(opt.value)}
      use:tooltip={opt.label}
    >
      {#if loading && isSelected}
        <Icon icon="mdi:loading" width={currentSize.icon} class="animate-spin" />
      {:else if opt.icon}
        <Icon icon={opt.icon} width={currentSize.icon} />
      {/if}
      {#if !iconOnly}
        <span class="select-none">{opt.label}</span>
      {/if}
    </button>
  {/each}
</div>
