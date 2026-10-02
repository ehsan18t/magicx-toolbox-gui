<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<div class="flex shrink-0 flex-wrap justify-end gap-2 border-t border-border bg-background px-6 py-4 {className}">
  {@render children()}
</div>
