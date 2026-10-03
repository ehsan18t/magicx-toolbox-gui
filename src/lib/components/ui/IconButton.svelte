<script lang="ts" module>
  import type { IconName, IconSize } from "$lib/design";
  import type { IconButtonSize } from "./variants";

  const GLYPH: Record<IconButtonSize, IconSize> = { xs: "xs", sm: "md", md: "lg" };
</script>

<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { iconButton, type IconButtonVariants } from "./variants";

  // The name is `label`, else the tooltip; given both, the tooltip becomes the description.
  type Props = Omit<HTMLButtonAttributes, "children" | "class" | "aria-label" | "aria-labelledby"> & {
    icon: IconName;
    size?: IconButtonSize;
    /** Swaps the icon for a spinner and disables the button. */
    loading?: boolean;
    /** Shows `tone` while on, e.g. an open panel or a starred item; pair with aria-pressed or aria-expanded. */
    active?: boolean;
    tone?: IconButtonVariants["tone"];
    class?: string;
  } & ({ label: string; tooltip?: string } | { label?: string; tooltip: string });

  let {
    icon,
    size = "md",
    loading = false,
    active = false,
    tone,
    label,
    tooltip,
    disabled,
    class: className,
    ...rest
  }: Props = $props();
</script>

<button
  type="button"
  class={iconButton({ size, active, tone, class: className })}
  disabled={disabled || loading}
  aria-busy={loading}
  aria-label={label ?? tooltip}
  use:tooltipAction={tooltip}
  {...rest}
>
  <Icon icon={loading ? "mdi:loading" : icon} size={GLYPH[size]} class={loading ? "animate-spin" : undefined} />
</button>
