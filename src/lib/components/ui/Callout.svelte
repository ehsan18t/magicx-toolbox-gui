<script lang="ts" module>
  import type { IconSize } from "$lib/design";
  import type { CalloutVariants } from "./variants";

  const GLYPH: Record<NonNullable<CalloutVariants["density"]>, IconSize> = {
    compact: "xs",
    note: "md",
    banner: "lg",
    panel: "lg",
  };
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { callout } from "./variants";

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "class">, CalloutVariants {
    /** Leading icon in the tone's colour, top-aligned with the first line. */
    icon?: IconName;
    class?: string | undefined;
    children: Snippet;
  }

  let { tone, density = "banner", bordered, icon, class: className, children, ...rest }: Props = $props();

  const styles = $derived(callout({ tone, density, bordered, withIcon: !!icon }));
</script>

<div class={styles.base({ class: className })} {...rest}>
  {#if icon}
    <Icon {icon} size={GLYPH[density]} class={styles.icon()} />
  {/if}
  {@render children()}
</div>
