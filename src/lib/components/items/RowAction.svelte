<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import { button } from "$lib/components/ui/variants";
  import { cn } from "$lib/utils/cn";
  import type { HTMLButtonAttributes } from "svelte/elements";

  // `aria-label` defaults to `label`: collapsed, the label is hidden.
  interface Props extends Pick<HTMLButtonAttributes, "aria-label" | "disabled"> {
    icon: IconName;
    label: string;
    tooltip?: string;
    /** Icon only in a narrow row. */
    collapses?: boolean;
    onclick: () => void;
  }

  let { icon, label, tooltip, collapses = false, onclick, ...rest }: Props = $props();
</script>

<button
  type="button"
  class={button({
    variant: "ghost",
    size: "sm",
    class: cn("shrink-0 px-2", collapses && "@max-item-row:w-7 @max-item-row:px-0"),
  })}
  aria-label={label}
  use:tooltipAction={tooltip}
  {onclick}
  {...rest}
>
  <Icon {icon} size="sm" class="shrink-0" />
  <span class={collapses ? "@max-item-row:hidden" : undefined}>{label}</span>
</button>
