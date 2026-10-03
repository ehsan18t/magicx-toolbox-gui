<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import type { IconName, IconSize } from "$lib/design";
  import { callout, type CalloutVariants } from "./variants";

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "class">, CalloutVariants {
    /** Leading icon in the tone's colour, top-aligned with the first line. */
    icon?: IconName;
    /** Defaults to the density's size. */
    iconSize?: IconSize;
    class?: string;
    children: Snippet;
  }

  let { tone, density = "banner", bordered, icon, iconSize, class: className, children, ...rest }: Props = $props();

  const styles = $derived(callout({ tone, density, bordered, withIcon: !!icon }));
</script>

<div class={styles.base({ class: className })} {...rest}>
  {#if icon}
    <Icon {icon} size={iconSize ?? (density === "compact" ? "xs" : "lg")} class={styles.icon()} />
  {/if}
  {@render children()}
</div>
