<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { glide } from "$lib/utils/motion";
  import { cn } from "@/utils";

  export interface SegmentOption {
    value: number;
    label: string;
    /** Iconify name, e.g. 'mdi:check' */
    icon?: string;
    disabled?: boolean;
    /** Tooltip in place of the label, e.g. why the segment is disabled. */
    tip?: string;
  }

  interface Props {
    value: number;
    options: SegmentOption[];
    pending?: boolean;
    loading?: boolean;
    disabled?: boolean;
    iconOnly?: boolean;
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
  // Only long labels give up width, so a short sibling is never cut to make room for them.
  const longestIndex = $derived(
    options.reduce((best, o, i) => (o.label.length > options[best].label.length ? i : best), 0),
  );
  const shrinks = (i: number) => i === longestIndex || options[i].label.length > 16;
  // With nothing selected the group still needs one tab stop.
  const tabStopIndex = $derived(selectedIndex >= 0 ? selectedIndex : options.findIndex((o) => !o.disabled));

  // The pill is the selected segment's ::before, free at mount; a change glides it over from the old segment.
  let group = $state<HTMLElement | null>(null);
  let previousIndex = -1;
  $effect(() => {
    const from = previousIndex;
    previousIndex = selectedIndex;
    if (from < 0 || selectedIndex < 0 || from === selectedIndex || !group) return;
    const segments = group.querySelectorAll<HTMLElement>("[role='radio']");
    if (segments[from] && segments[selectedIndex]) glide(segments[from], segments[selectedIndex]);
  });

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
  bind:this={group}
  role="radiogroup"
  aria-label={label}
  tabindex="-1"
  class={cn(
    "relative isolate inline-flex max-w-full items-center gap-0.5 rounded-md border p-0.5 transition-colors",
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
        "relative inline-flex items-center justify-center gap-1.5 rounded font-medium whitespace-nowrap",
        shrinks(i) ? "min-w-0" : "shrink-0",
        "outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed",
        iconOnly ? currentSize.segmentIconOnly : currentSize.segment,
        isSelected
          ? cn(
              "before:absolute before:inset-0 before:-z-1 before:origin-left before:rounded before:shadow-sm before:transition-colors",
              pending ? "text-warning-foreground before:bg-warning" : "text-accent-foreground before:bg-accent",
            )
          : cn(
              "text-foreground-muted",
              opt.disabled && "opacity-40",
              !disabled && !loading && !opt.disabled && "cursor-pointer hover:bg-muted hover:text-foreground",
            ),
      )}
      onclick={() => handleClick(opt.value)}
      use:tooltip={opt.tip ?? opt.label}
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
