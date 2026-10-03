<script lang="ts">
  import { SKELETON_SOFT } from "$lib/design";
  import { cn } from "$lib/utils/cn";

  interface Props {
    /** A secondary line, a step fainter. */
    soft?: boolean;
    /** Off inside a placeholder that pulses as a whole. */
    pulse?: boolean;
    class?: string;
    style?: string;
  }

  let { soft = false, pulse = true, class: className, style }: Props = $props();
</script>

<div class={cn("rounded", soft ? SKELETON_SOFT : "bg-muted", pulse && "animate-pulse", className)} {style}></div>
