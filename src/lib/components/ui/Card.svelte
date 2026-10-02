<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLAttributes<HTMLElement>, "class"> {
    class?: string;
    as?: "div" | "section";
    children: Snippet;
  }

  let { as = "div", class: className, children, ...rest }: Props = $props();
</script>

<svelte:element this={as} class={cn("rounded-lg border border-border bg-card", className)} {...rest}>
  {@render children()}
</svelte:element>
