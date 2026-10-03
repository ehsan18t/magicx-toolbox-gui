<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";

  interface Props {
    class?: string;
    children: Snippet;
  }

  let { class: className, children }: Props = $props();
</script>

<div class={cn("flex shrink-0 flex-wrap justify-end gap-2 border-t border-border bg-background px-6 py-4", className)}>
  {@render children()}
</div>
