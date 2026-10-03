<script lang="ts">
  import { HEADING } from "$lib/design";
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import Card from "./Card.svelte";

  interface Props {
    title: string;
    /** `outside`: the title sits above the card, as a page section heading. */
    heading?: "inside" | "outside";
    /** Trailing header controls, e.g. a refresh button. */
    actions?: Snippet;
    class?: string;
    children: Snippet;
  }

  let { title, heading = "inside", actions, class: className, children }: Props = $props();

  const titleId = $props.id();
</script>

{#if heading === "outside"}
  <section class={className} aria-labelledby={titleId}>
    <h2 id={titleId} class={["m-0 mb-2 text-foreground", HEADING.item]}>{title}</h2>
    <Card>{@render children()}</Card>
  </section>
{:else}
  <Card as="section" class={cn("overflow-hidden", className)} aria-labelledby={titleId}>
    <div
      class={[
        "border-b border-border",
        actions ? "flex items-center justify-between gap-3 py-1 pr-1 pl-3" : "px-3 py-2",
      ]}
    >
      <h2 id={titleId} class={["m-0", HEADING.group]}>{title}</h2>
      {@render actions?.()}
    </div>
    {@render children()}
  </Card>
{/if}
