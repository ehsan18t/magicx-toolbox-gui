<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="flex shrink-0 flex-wrap justify-end gap-2 border-t border-border bg-background px-6 py-4">
  {@render children()}
</div>
