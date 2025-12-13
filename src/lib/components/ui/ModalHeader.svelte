<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
  }

  const { children, class: className = "" }: Props = $props();
</script>

<div class="flex items-start justify-between gap-3 border-b border-border px-5 py-4 {className}">
  {@render children()}
</div>
