<script lang="ts" module>
  import { ICON_SIZE } from "./icon";
  import type { IconButtonSize } from "./variants";

  const GLYPH: Record<IconButtonSize, number> = { xs: ICON_SIZE.sm, sm: ICON_SIZE.md, md: ICON_SIZE.lg };
</script>

<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon, type IconName } from "$lib/components/shared";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { iconButton } from "./variants";

  // The name is `label`, else the tooltip; given both, the tooltip becomes the description.
  type Props = Omit<HTMLButtonAttributes, "children" | "class" | "aria-label" | "aria-labelledby"> & {
    icon: IconName;
    size?: IconButtonSize;
    class?: string;
  } & ({ label: string; tooltip?: string } | { label?: string; tooltip: string });

  let { icon, size = "md", label, tooltip, class: className, ...rest }: Props = $props();
</script>

<button
  type="button"
  class={iconButton({ size, class: className })}
  aria-label={label ?? tooltip}
  use:tooltipAction={tooltip}
  {...rest}
>
  <Icon {icon} width={GLYPH[size]} />
</button>
