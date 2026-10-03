<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { ICON_SIZE, TONE_TEXT, type TextTone } from "$lib/components/ui";
  import { cn } from "$lib/utils/cn";

  interface Props {
    icon: string;
    label: string;
    /** Omitted: inherits the line's colour. */
    tone?: TextTone;
    tooltip?: string | null;
    spin?: boolean;
    size?: "sm" | "md";
    /** Cuts the label to the line's width instead of wrapping the line. */
    truncate?: boolean;
    class?: string;
  }

  let { icon, label, tone, tooltip, spin = false, size = "sm", truncate = false, class: className }: Props = $props();
</script>

<span
  class={cn("inline-flex items-center gap-1", truncate && "max-w-full", tone && TONE_TEXT[tone], className)}
  use:tooltipAction={tooltip}
>
  <Icon
    {icon}
    width={size === "sm" ? ICON_SIZE.xs : ICON_SIZE.sm}
    class={spin ? "shrink-0 animate-spin" : "shrink-0"}
  />
  {#if truncate}<span class="truncate">{label}</span>{:else}{label}{/if}
  <!-- The span is not focusable, so the tooltip is read from here instead. -->
  {#if tooltip}<span class="sr-only">({tooltip})</span>{/if}
</span>
