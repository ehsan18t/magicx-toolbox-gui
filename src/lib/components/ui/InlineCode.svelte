<script lang="ts">
  import { INLINE_CODE } from "$lib/design";
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";

  let { class: className, children }: { class?: string; children: Snippet } = $props();
</script>

<code class={cn(INLINE_CODE, className)}>{@render children()}</code>
