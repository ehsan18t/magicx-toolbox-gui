<script lang="ts">
  import { cn } from "$lib/utils/cn";

  interface Props {
    value: number;
    class?: string;
  }

  let { value, class: className }: Props = $props();
</script>

<span class={cn("text-xs text-foreground-subtle tabular-nums", className)}>{value}</span>
