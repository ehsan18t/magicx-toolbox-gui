<script lang="ts" module>
  import type { CalloutVariants } from "./variants";

  type Density = NonNullable<CalloutVariants["density"]>;

  const ICON_LAYOUT: Record<Density, { size: number; row: string; nudge: string }> = {
    compact: { size: 14, row: "gap-2", nudge: "mt-px" },
    banner: { size: 18, row: "gap-3", nudge: "mt-0.5" },
    panel: { size: 18, row: "gap-3", nudge: "mt-0.5" },
  };
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { TONE_TEXT } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { callout } from "./variants";

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "class">, CalloutVariants {
    class?: string;
    /** Leading icon in the tone's colour, top-aligned with the first line. */
    icon?: string;
    children: Snippet;
  }

  let { tone = "neutral", density = "banner", icon, class: className, children, ...rest }: Props = $props();

  const layout = $derived(ICON_LAYOUT[density]);
</script>

<div class={callout({ tone, density, class: [icon && `flex items-start ${layout.row}`, className] })} {...rest}>
  {#if icon}
    <Icon {icon} width={layout.size} class="{layout.nudge} shrink-0 {TONE_TEXT[tone]}" />
  {/if}
  {@render children()}
</div>
