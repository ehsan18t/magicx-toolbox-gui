<script lang="ts" module>
  import type { IconButtonSize } from "./variants";

  const ICON_SIZE: Record<IconButtonSize, number> = { xs: 14, sm: 16, md: 18 };
</script>

<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { iconButton } from "./variants";

  type Props = Omit<HTMLButtonAttributes, "children" | "class"> & {
    class?: string;
    icon: string;
    size?: IconButtonSize;
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
  <Icon {icon} width={ICON_SIZE[size]} />
</button>
