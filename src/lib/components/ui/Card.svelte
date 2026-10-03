<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { card, type CardElevation } from "./variants";

  interface Props extends Omit<HTMLAttributes<HTMLElement>, "class"> {
    as?: "div" | "section";
    elevation?: CardElevation;
    class?: string;
    children: Snippet;
  }

  let { as = "div", elevation, class: className, children, ...rest }: Props = $props();
</script>

<svelte:element this={as} class={card({ elevation, class: className })} {...rest}>
  {@render children()}
</svelte:element>
