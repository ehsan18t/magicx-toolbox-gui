<script lang="ts">
  import { tooltip as withTooltip } from "$lib/attachments/tooltip.svelte";
  import { Icon } from "$lib/components/shared";
  import { type IconName, type TextTone, TONE_TEXT } from "$lib/design";
  import { cn } from "$lib/utils/cn";

  interface Props {
    icon: IconName;
    label: string;
    /** Omitted: inherits the line's color. */
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
  {@attach withTooltip(() => tooltip)}
>
  <Icon {icon} size={size === "sm" ? "2xs" : "xs"} class={spin ? "shrink-0 animate-spin" : "shrink-0"} />
  {#if truncate}<span class="truncate">{label}</span>{:else}{label}{/if}
  <!-- The span is not focusable, so the tooltip is read from here instead. -->
  {#if tooltip}<span class="sr-only">({tooltip})</span>{/if}
</span>
