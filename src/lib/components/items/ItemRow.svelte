<script lang="ts" module>
  import { EMPHASIS_BORDER, type FillTone, HEADING, TONE_FILL, TONE_WASH } from "$lib/design";

  export type RowStripe = FillTone;

  const BORDER = { ...EMPHASIS_BORDER, none: "border-border hover:border-border-hover" };
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Callout, HighlightedText, IconButton, indicator, META_LINE } from "$lib/components/ui";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import type { SearchResult } from "$lib/stores/search.svelte";
  import type { ItemKind } from "$lib/types";
  import { cn } from "$lib/utils/cn";
  import { infoExcerpt } from "$lib/utils/excerpt";
  import { expand } from "$lib/utils/motion";
  import { rowDomId } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { searchHighlight } from "./searchHighlight.svelte";

  interface Props extends Omit<HTMLAttributes<HTMLElement>, "id" | "class" | "title" | "onclick"> {
    kind: ItemKind;
    id: string;
    title: string;
    description: string;
    /** The Details markdown; a match found only here shows as an excerpt. */
    info?: string | null;
    /** Highlight ranges; defaults to the page filter's. */
    match?: SearchResult | null;
    stripe?: RowStripe | null;
    emphasis?: keyof typeof BORDER;
    error?: string;
    ondismisserror: () => void;
    /** Mouse only: the row's Details action is its keyboard path. */
    onclick?: (e: MouseEvent) => void;
    /** Beside the title. */
    control?: Snippet;
    notices?: Snippet;
    meta: Snippet;
    /** After the meta items, e.g. the category in search results. */
    context?: Snippet;
    actions: Snippet;
  }

  let {
    kind,
    id,
    title,
    description,
    info,
    match,
    stripe = null,
    emphasis = "none",
    error,
    ondismisserror,
    onclick,
    control,
    notices,
    meta,
    context,
    actions,
    ...rest
  }: Props = $props();

  const ranges = $derived(match ?? pageFilterStore.match(id));
  const excerpt = $derived(
    info && ranges?.infoRanges.length && !ranges.nameRanges.length && !ranges.descriptionRanges.length
      ? infoExcerpt(info, ranges.infoRanges)
      : null,
  );

  let rowEl = $state<HTMLElement | null>(null);
  const highlight = searchHighlight(
    () => id,
    () => rowEl,
  );
</script>

<!-- Out of the tab order, yet focusable: a search "Go to" focuses the row it scrolls to. -->
<article
  id={rowDomId(kind, id)}
  bind:this={rowEl}
  tabindex="-1"
  class={cn(
    "@container relative flex min-w-0 flex-col overflow-hidden rounded-lg border bg-card transition-colors",
    BORDER[emphasis],
    onclick && "cursor-pointer",
    highlight.active && "animate-highlight",
  )}
  {onclick}
  {...rest}
>
  <span
    class={indicator({ shape: "stripe", class: ["transition-colors", stripe ? TONE_FILL[stripe] : "bg-transparent"] })}
    aria-hidden="true"
  ></span>

  <div class="flex flex-1 flex-col gap-2.5 py-3 pr-3 pl-4">
    <div class="grid grid-cols-item-row items-center gap-x-6 gap-y-1 @max-item-row:grid-cols-1 @max-item-row:gap-y-2">
      <h3 class={cn(HEADING.item, "m-0 leading-snug wrap-break-word text-foreground")}>
        {#if ranges}<HighlightedText text={title} ranges={ranges.nameRanges} />{:else}{title}{/if}
      </h3>

      {@render control?.()}

      <p class="col-span-full m-0 text-ui leading-snug text-foreground-muted">
        {#if ranges}<HighlightedText text={description} ranges={ranges.descriptionRanges} />{:else}{description}{/if}
      </p>

      {#if excerpt}
        <p class="col-span-full m-0 flex min-w-0 items-center gap-1 text-xs leading-snug text-foreground-muted">
          <Icon icon="mdi:text-search" size="2xs" class="shrink-0" />
          <span class="sr-only">Matched in details:</span>
          <HighlightedText text={excerpt.text} ranges={excerpt.ranges} class="min-w-0 truncate" />
        </p>
      {/if}
    </div>

    {@render notices?.()}

    {#if error}
      <div transition:expand>
        <Callout tone="error" density="compact" icon="mdi:alert-circle" class="text-error" role="alert">
          <span class="min-w-0 flex-1 wrap-break-word">{error}</span>
          <IconButton
            size="xs"
            icon="mdi:close"
            label="Dismiss error"
            class={TONE_WASH.error}
            onclick={ondismisserror}
          />
        </Callout>
      </div>
    {/if}

    <div class={["mt-auto", META_LINE]}>
      {@render meta()}
      {@render context?.()}
      <div class="ml-auto flex items-center gap-0.5">
        {@render actions()}
      </div>
    </div>
  </div>
</article>
