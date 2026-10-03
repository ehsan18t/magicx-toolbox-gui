<script lang="ts">
  import { tooltip as withTooltip } from "$lib/attachments/tooltip.svelte";
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import DisabledReason from "./DisabledReason.svelte";
  import { blockedReason, reasonAttrs } from "./disabledReason";
  import Spinner from "./Spinner.svelte";
  import { button, type ButtonVariants } from "./variants";

  interface Props extends Omit<HTMLButtonAttributes, "class">, ButtonVariants {
    /** Leading icon, replaced by a spinner while `loading`. */
    icon?: IconName;
    loading?: boolean;
    /** Shows on hover and focus; while disabled, it is the reason and the button stays focusable. */
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
    onclick,
    ...rest
  }: Props = $props();

  const reasonId = $props.id();
  const reason = $derived(blockedReason(disabled, loading, tooltip));

  function handleClick(e: MouseEvent & { currentTarget: EventTarget & HTMLButtonElement }) {
    if (reason) e.preventDefault();
    else onclick?.(e);
  }
</script>

<!-- A disabled button still gets mouse events in Chromium, so the tooltip needs no wrapper. -->
<button
  type="button"
  class={button({ variant, size, tone, class: className })}
  disabled={(disabled && !reason) || loading}
  {...reasonAttrs(reason, reasonId)}
  aria-busy={loading}
  {@attach withTooltip(() => tooltip)}
  {...rest}
  onclick={handleClick}
>
  {#if loading}
    <Spinner size="md" tone="current" />
  {:else if icon}
    <Icon {icon} size="md" />
  {/if}
  {@render children()}
  <DisabledReason id={reasonId} {reason} />
</button>
