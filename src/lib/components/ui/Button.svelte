<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { ICON_SIZE } from "./icon";
  import { button, type ButtonVariants } from "./variants";

  interface Props extends Omit<HTMLButtonAttributes, "class">, ButtonVariants {
    class?: string;
    loading?: boolean;
    children: Snippet;
  }

  let { variant, size, loading = false, disabled, class: className, children, ...rest }: Props = $props();
</script>

<button
  type="button"
  class={button({ variant, size, class: className })}
  disabled={disabled || loading}
  aria-busy={loading}
  {...rest}
>
  {#if loading}
    <Icon icon="mdi:loading" width={ICON_SIZE.md} class="animate-spin" />
  {/if}
  {@render children()}
</button>
