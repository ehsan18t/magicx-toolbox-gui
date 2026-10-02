<script lang="ts" module>
  const SHRINK_LABEL_CHARS = 16;
</script>

<script lang="ts" generics="T extends string | number">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { glide } from "$lib/utils/motion";
  import type { SegmentOption } from "./types";

  interface Props {
    value: T;
    options: SegmentOption<T>[];
    label: string;
    pending?: boolean;
    loading?: boolean;
    disabled?: boolean;
    onchange?: (value: T) => void;
  }

  let { value, options, label, pending = false, loading = false, disabled = false, onchange }: Props = $props();

  const selectedIndex = $derived(options.findIndex((o) => o.value === value));
  // Only long labels give up width, so a short sibling is never cut to make room for them.
  const longestIndex = $derived(
    options.reduce((best, o, i) => (o.label.length > options[best].label.length ? i : best), 0),
  );
  const shrinks = (i: number) => i === longestIndex || options[i].label.length > SHRINK_LABEL_CHARS;
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

  function handleClick(optValue: T) {
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
  )}
  onkeydown={handleKeydown}
>
  {#each options as opt, i (opt.value)}
    {@const isSelected = opt.value === value}
    <button
      type="button"
      role="radio"
      aria-checked={isSelected}
      tabindex={i === tabStopIndex ? 0 : -1}
      disabled={disabled || loading || opt.disabled}
      class={cn(
        "relative inline-flex items-center justify-center gap-1.5 rounded font-medium whitespace-nowrap",
        shrinks(i) ? "min-w-0" : "shrink-0",
        "outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed",
        "h-7 px-3 text-ui",
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
        <Icon icon="mdi:loading" width={14} class="shrink-0 animate-spin" />
      {:else if opt.icon}
        <Icon icon={opt.icon} width={14} class="shrink-0" />
      {/if}
      <span class="truncate">{opt.label}</span>
    </button>
  {/each}
</div>
