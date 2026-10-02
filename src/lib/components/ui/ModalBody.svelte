<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<div class="min-h-0 flex-1 overflow-y-auto px-6 py-5 {className}">
  {@render children()}
</div>
