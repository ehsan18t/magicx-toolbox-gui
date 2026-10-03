<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { button, type ButtonVariants } from "./variants";

  interface Props extends Omit<HTMLButtonAttributes, "class">, ButtonVariants {
    /** Leading icon, replaced by a spinner while `loading`. */
    icon?: IconName;
    loading?: boolean;
    /** Shows on hover even while disabled, e.g. why the button is disabled. */
    tooltip?: string | null;
    class?: string;
    children: Snippet;
  }

  let {
    variant,
    size,
    tone,
    icon,
    loading = false,
    tooltip,
    disabled,
    class: className,
    children,
    ...rest
  }: Props = $props();
</script>

<!-- A disabled button still gets mouse events in Chromium, so the tooltip needs no wrapper. -->
<button
  type="button"
  class={button({ variant, size, tone, class: className })}
  disabled={disabled || loading}
  aria-busy={loading}
  use:tooltipAction={tooltip}
  {...rest}
>
  {#if loading}
    <Icon icon="mdi:loading" size="md" class="animate-spin" />
  {:else if icon}
    <Icon {icon} size="md" />
  {/if}
  {@render children()}
</button>
