<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { toggleChip, type ToggleChipVariants } from "./variants";

  // A pressable chip; the caller sets aria-pressed (a filter) or aria-expanded (a disclosure).
  interface Props extends Omit<HTMLButtonAttributes, "class">, ToggleChipVariants {
    icon?: IconName;
    class?: string;
    children: Snippet;
  }

  let { size, variant, tone, icon, class: className, children, ...rest }: Props = $props();
</script>

<button type="button" class={toggleChip({ size, variant, tone, class: className })} {...rest}>
  {#if icon}<Icon {icon} size={size === "md" ? "md" : "2xs"} class="shrink-0" />{/if}
  {@render children()}
</button>
