<script lang="ts">
  import { tooltip as withTooltip } from "$lib/attachments/tooltip.svelte";
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { toggleChip, type ToggleChipVariants } from "./variants";

  // A pressable chip; the caller sets aria-pressed (a filter) or aria-expanded (a disclosure).
  interface Props extends Omit<HTMLButtonAttributes, "class">, ToggleChipVariants {
    icon?: IconName;
    /** Shows on hover even while disabled, e.g. why the chip is held. */
    tooltip?: string | null;
    class?: string | undefined;
    children: Snippet;
  }

  let { size, variant, tone, icon, tooltip, class: className, children, ...rest }: Props = $props();
</script>

<button
  type="button"
  class={toggleChip({ size, variant, tone, class: className })}
  {@attach withTooltip(() => tooltip)}
  {...rest}
>
  {#if icon}<Icon {icon} size={size === "md" ? "md" : "2xs"} class="shrink-0" />{/if}
  {@render children()}
</button>
