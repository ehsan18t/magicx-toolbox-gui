<script lang="ts" module>
  import type { Tone } from "$lib/components/ui";

  export type RowStripe = Extract<Tone, "accent" | "warning" | "error">;

  const STRIPE: Record<RowStripe, string> = { accent: "bg-accent", warning: "bg-warning", error: "bg-error" };

  const BORDER = {
    selected: "border-accent/70",
    pending: "border-warning/45",
    none: "border-border hover:border-border-hover",
  };
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Callout, HighlightedText, ICON_SIZE } from "$lib/components/ui";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import type { SearchResult } from "$lib/stores/search.svelte";
  import type { ItemKind } from "$lib/types";
  import { cn } from "$lib/utils/cn";
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

  let rowEl = $state<HTMLElement | null>(null);
  const highlight = searchHighlight(
    () => id,
    () => rowEl,
  );
</script>

<article
  id={rowDomId(kind, id)}
  bind:this={rowEl}
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
    class="absolute top-3 bottom-3 left-0 w-0.75 rounded-r-full transition-colors {stripe
      ? STRIPE[stripe]
      : 'bg-transparent'}"
    aria-hidden="true"
  ></span>

  <div class="flex flex-1 flex-col gap-2.5 py-3 pr-3 pl-4">
    <div class="grid grid-cols-item-row items-center gap-x-6 gap-y-1 @max-item-row:grid-cols-1 @max-item-row:gap-y-2">
      <h3 class="m-0 text-sm leading-snug font-semibold wrap-break-word text-foreground">
        {#if ranges}<HighlightedText text={title} ranges={ranges.nameRanges} />{:else}{title}{/if}
      </h3>

      {@render control?.()}

      <p class="col-span-full m-0 text-ui leading-snug text-foreground-muted">
        {#if ranges}<HighlightedText text={description} ranges={ranges.descriptionRanges} />{:else}{description}{/if}
      </p>
    </div>

    {@render notices?.()}

    {#if error}
      <div transition:expand>
        <Callout tone="error" density="compact" icon="mdi:alert-circle" class="text-error" role="alert">
          <span class="min-w-0 flex-1 wrap-break-word">{error}</span>
          <button
            type="button"
            class="flex shrink-0 cursor-pointer rounded p-0.5 text-error/70 hover:bg-error/10 hover:text-error"
            onclick={ondismisserror}
            aria-label="Dismiss error"
          >
            <Icon icon="mdi:close" width={ICON_SIZE.sm} />
          </button>
        </Callout>
      </div>
    {/if}

    <div class="mt-auto flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
      {@render meta()}
      {@render context?.()}
      <div class="ml-auto flex items-center gap-0.5">
        {@render actions()}
      </div>
    </div>
  </div>
</article>
