<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import Card from "./Card.svelte";

  interface Props {
    title: string;
    /** Trailing header controls, e.g. a refresh button. */
    actions?: Snippet;
    class?: string;
    children: Snippet;
  }

  let { title, actions, class: className, children }: Props = $props();

  const titleId = $props.id();
</script>

<Card as="section" class={cn("overflow-hidden", className)} aria-labelledby={titleId}>
  <div
    class={["border-b border-border", actions ? "flex items-center justify-between gap-3 py-1 pr-1 pl-3" : "px-3 py-2"]}
  >
    <h2 id={titleId} class="m-0 text-ui font-semibold">{title}</h2>
    {@render actions?.()}
  </div>
  {@render children()}
</Card>
