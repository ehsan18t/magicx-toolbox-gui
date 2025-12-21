<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { actionButton, counterBadge, type ActionButtonVariants, type CounterBadgeVariants } from "./variants";

  interface Props extends HTMLButtonAttributes {
    /** Button intent/style */
    intent?: ActionButtonVariants["intent"];
    /** Button size */
    size?: ActionButtonVariants["size"];
    /** Whether button has active state (e.g., pending changes) */
    active?: boolean;
    /** Icon name (mdi icon) */
    icon?: string;
    /** Show loading spinner instead of icon */
    loading?: boolean;
    /** Optional badge count */
    badgeCount?: number;
    /** Badge variant */
    badgeVariant?: CounterBadgeVariants["variant"];
    /** Hide label on small screens */
    hideLabel?: boolean;
    /** Tooltip text */
    tooltip?: string;
    /** Additional classes */
    class?: string;
    /** Button content */
    children?: Snippet;
  }

  const {
    intent = "default",
    size = "md",
    active = false,
    icon,
    loading = false,
    badgeCount,
    badgeVariant = "warning",
    hideLabel = true,
    tooltip,
    disabled,
    class: className = "",
    children,
    ...rest
  }: Props = $props();
</script>

<button
  type="button"
  class={actionButton({ intent, size, active, class: className })}
  disabled={disabled || loading}
  use:tooltipAction={tooltip}
  {...rest}
>
  {#if loading}
    <Icon icon="mdi:loading" width="18" class="animate-spin" />
  {:else if icon}
    <Icon {icon} width="18" />
  {/if}
  {#if children}
    <span class={hideLabel ? "hidden sm:inline" : ""}>
      {@render children()}
    </span>
  {/if}
  {#if badgeCount !== undefined && badgeCount > 0}
    <span class={counterBadge({ variant: badgeVariant })}>
      {badgeCount}
    </span>
  {/if}
</button>
