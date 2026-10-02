<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";

  interface Props {
    class?: string;
    children: Snippet;
  }

  let { class: className, children }: Props = $props();
</script>

<div class={cn("min-h-0 flex-1 overflow-y-auto px-6 py-5", className)}>
  {@render children()}
</div>
