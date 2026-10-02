<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { TONE_TEXT, type Tone } from "$lib/utils/tweakPresentation";

  interface Props {
    icon: string;
    label: string;
    /** Omitted: inherits the line's colour. */
    tone?: Tone;
    tip?: string | null;
    spin?: boolean;
    size?: "sm" | "md";
    /** Cuts the label to the line's width instead of wrapping the line. */
    truncate?: boolean;
    class?: string;
  }

  let { icon, label, tone, tip, spin = false, size = "sm", truncate = false, class: className }: Props = $props();
</script>

<span
  class={cn("inline-flex items-center gap-1", truncate && "max-w-full", tone && TONE_TEXT[tone], className)}
  use:tooltip={tip}
>
  <Icon {icon} width={size === "sm" ? 13 : 14} class={spin ? "shrink-0 animate-spin" : "shrink-0"} />
  {#if truncate}<span class="truncate">{label}</span>{:else}{label}{/if}
</span>
