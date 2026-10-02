<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { button, type ButtonVariants } from "./variants";

  interface Props extends HTMLButtonAttributes {
    variant?: ButtonVariants["variant"];
    size?: ButtonVariants["size"];
    loading?: boolean;
    class?: string;
    children: Snippet;
  }

  let {
    variant = "secondary",
    size = "md",
    loading = false,
    disabled,
    class: className = "",
    children,
    ...rest
  }: Props = $props();
</script>

<button
  class={button({ variant, size, class: className })}
  disabled={disabled || loading}
  aria-busy={loading}
  {...rest}
>
  {#if loading}
    <Icon icon="mdi:loading" width={16} class="animate-spin" />
  {/if}
  {@render children()}
</button>
