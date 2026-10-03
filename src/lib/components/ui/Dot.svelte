<script lang="ts">
  import { cn } from "$lib/utils/cn";

  interface Props {
    /** A background class, e.g. from TONE_FILL. */
    fill: string;
    class?: string;
  }

  let { fill, class: className }: Props = $props();
</script>

<span class={cn("h-2 w-2 rounded-full", fill, className)} aria-hidden="true"></span>
