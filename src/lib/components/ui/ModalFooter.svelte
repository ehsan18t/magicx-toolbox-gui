<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<div class="bg-muted/30 flex justify-end gap-2 border-t border-border px-5 py-3 {className}">
  {@render children()}
</div>
