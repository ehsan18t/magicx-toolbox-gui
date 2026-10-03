<script lang="ts" module>
  const SHRINK_LABEL_CHARS = 16;
</script>

<script lang="ts" generics="T extends string | number">
  import { textIfCut, tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { glide } from "$lib/utils/motion";
  import { PENDING_TINT } from "$lib/design";
  import { nextEnabledIndex, radioKeyIndex } from "./listNav";
  import Spinner from "./Spinner.svelte";
  import type { SegmentOption } from "./types";
  import { DIMMED } from "./variants";

  interface Props {
    /** Null selects nothing, e.g. while the current state matches no option. */
    value: T | null;
    options: SegmentOption<T>[];
    label: string;
    pending?: boolean;
    loading?: boolean;
    disabled?: boolean;
    class?: string;
    onchange?: (value: T) => void;
  }

  let {
    value,
    options,
    label,
    pending = false,
    loading = false,
    disabled = false,
    class: className,
    onchange,
  }: Props = $props();

  const selectedIndex = $derived(options.findIndex((o) => o.value === value));
  // Only long labels give up width, so a short sibling is never cut to make room for them.
  const longestIndex = $derived(
    options.reduce((best, o, i) => (o.label.length > options[best].label.length ? i : best), 0),
  );
  const shrinks = (i: number) => i === longestIndex || options[i].label.length > SHRINK_LABEL_CHARS;
  // With nothing selected the group still needs one tab stop.
  const tabStopIndex = $derived(selectedIndex >= 0 ? selectedIndex : nextEnabledIndex(options, -1, 1));

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

  function choose(opt: SegmentOption<T>) {
    if (!disabled && !loading && opt.value !== value) onchange?.(opt.value);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (disabled || loading || !group) return;
    const segments = [...group.querySelectorAll<HTMLElement>("[role='radio']")];
    const focused = segments.indexOf(document.activeElement as HTMLElement);
    const from = focused >= 0 ? focused : selectedIndex;

    const next = radioKeyIndex(options, from, e.key);
    if (next === null) return;
    e.preventDefault();
    if (next < 0) return;
    segments[next].focus();
    if (!options[next].confirms) choose(options[next]);
  }
</script>

<!-- Svelte requires a tabindex on a radiogroup that takes keys; -1 keeps the group itself out of the tab order. -->
<div
  bind:this={group}
  role="radiogroup"
  aria-label={label}
  tabindex="-1"
  class={cn(
    "relative isolate inline-flex max-w-full items-center gap-0.5 rounded-md border p-0.5 transition-colors",
    pending ? PENDING_TINT : "border-border bg-secondary",
    disabled && DIMMED,
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
      aria-disabled={loading || undefined}
      tabindex={i === tabStopIndex ? 0 : -1}
      disabled={disabled || opt.disabled}
      class={cn(
        "relative inline-flex h-7 items-center justify-center gap-1.5 rounded px-3 text-ui font-medium whitespace-nowrap",
        shrinks(i) ? "min-w-0" : "shrink-0",
        "outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed aria-disabled:cursor-wait",
        isSelected
          ? cn(
              "before:absolute before:inset-0 before:-z-1 before:origin-left before:rounded before:shadow-sm before:transition-colors",
              pending ? "text-warning-foreground before:bg-warning" : "text-accent-foreground before:bg-accent",
            )
          : cn(
              "text-foreground-muted",
              opt.disabled && !disabled && DIMMED,
              !disabled && !loading && !opt.disabled && "cursor-pointer hover:bg-muted hover:text-foreground",
            ),
      )}
      onclick={() => choose(opt)}
      use:tooltip={opt.tooltip ?? textIfCut(opt.label, (node) => node.lastElementChild)}
    >
      {#if loading && isSelected}
        <Spinner size="xs" tone="current" class="shrink-0" />
      {:else if opt.icon}
        <Icon icon={opt.icon} size="xs" class="shrink-0" />
      {/if}
      <span class="truncate">{opt.label}</span>
    </button>
  {/each}
</div>
