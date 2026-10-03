<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import type { IconName } from "$lib/design";
  import { Icon } from "$lib/components/shared";
  import { button } from "$lib/components/ui/variants";
  import { cn } from "$lib/utils/cn";

  interface Props {
    icon: IconName;
    label: string;
    /** Defaults to `label`. */
    ariaLabel?: string;
    tooltip?: string;
    disabled?: boolean;
    /** Icon only in a narrow row. */
    collapses?: boolean;
    onclick: () => void;
  }

  let { icon, label, ariaLabel = label, tooltip, disabled, collapses = false, onclick }: Props = $props();
</script>

<button
  type="button"
  class={button({
    variant: "ghost",
    size: "sm",
    class: cn("shrink-0 px-2", collapses && "@max-item-row:w-7 @max-item-row:px-0"),
  })}
  aria-label={ariaLabel}
  use:tooltipAction={tooltip}
  {disabled}
  {onclick}
>
  <Icon {icon} size="sm" class="shrink-0" />
  <span class={collapses ? "@max-item-row:hidden" : undefined}>{label}</span>
</button>
