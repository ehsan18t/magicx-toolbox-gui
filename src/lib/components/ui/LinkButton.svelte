<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { link, type LinkVariant } from "./variants";

  // An action that reads as a link; colour comes from the surrounding text unless `class` sets it.
  interface Props extends Omit<HTMLButtonAttributes, "class"> {
    variant?: LinkVariant;
    class?: string;
    children: Snippet;
  }

  let { variant, class: className, children, ...rest }: Props = $props();
</script>

<button type="button" class={link({ variant, class: ["border-0 bg-transparent p-0", className] })} {...rest}>
  {@render children()}
</button>
