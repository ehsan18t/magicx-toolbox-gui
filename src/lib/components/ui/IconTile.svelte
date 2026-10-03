<script lang="ts" module>
  import type { IconSize } from "$lib/design";
  import type { IconTileVariants } from "./variants";

  const GLYPH: Record<NonNullable<IconTileVariants["size"]>, IconSize> = {
    sm: "lg",
    md: "3xl",
    lg: "3xl",
    xl: "4xl",
    "2xl": "5xl",
    "3xl": "6xl",
    "4xl": "7xl",
  };
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import { iconTile } from "./variants";

  interface Props extends IconTileVariants {
    icon: IconName;
    /** Defaults to the tile's own glyph size. */
    iconSize?: IconSize;
    spin?: boolean;
    class?: string;
  }

  let { icon, size = "md", shape, tone, iconSize, spin = false, class: className }: Props = $props();
</script>

<span aria-hidden="true" class={iconTile({ size, shape, tone, class: className })}>
  <Icon {icon} size={iconSize ?? GLYPH[size]} class={spin ? "animate-spin" : undefined} />
</span>
