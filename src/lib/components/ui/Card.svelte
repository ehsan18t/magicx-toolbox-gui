<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { card, type CardVariants } from "./variants";

  interface Props extends Omit<HTMLAttributes<HTMLElement>, "class">, CardVariants {
    as?: "div" | "section";
    class?: string;
    children: Snippet;
  }

  let { as = "div", radius, elevation, class: className, children, ...rest }: Props = $props();
</script>

<svelte:element this={as} class={card({ radius, elevation, class: className })} {...rest}>
  {@render children()}
</svelte:element>
