<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { ICON_SIZE } from "./icon";
  import { callout, type CalloutVariants } from "./variants";

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "class">, CalloutVariants {
    /** Leading icon in the tone's colour, top-aligned with the first line. */
    icon?: string;
    class?: string;
    children: Snippet;
  }

  let { tone, density = "banner", icon, class: className, children, ...rest }: Props = $props();

  const styles = $derived(callout({ tone, density, withIcon: !!icon }));
</script>

<div class={styles.base({ class: className })} {...rest}>
  {#if icon}
    <Icon {icon} width={density === "compact" ? ICON_SIZE.sm : ICON_SIZE.lg} class={styles.icon()} />
  {/if}
  {@render children()}
</div>
