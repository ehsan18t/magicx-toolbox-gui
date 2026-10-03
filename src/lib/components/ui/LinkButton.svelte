<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { link, type LinkVariants } from "./variants";

  interface Props extends Omit<HTMLButtonAttributes, "class">, LinkVariants {
    class?: string;
    children: Snippet;
  }

  let { variant, tone, class: className, children, ...rest }: Props = $props();
</script>

<button type="button" class={link({ variant, tone, class: ["border-0 bg-transparent p-0", className] })} {...rest}>
  {@render children()}
</button>
