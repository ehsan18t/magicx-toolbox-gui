<script lang="ts">
  import { cn } from "$lib/utils/cn";

  // Indeterminate and decorative: pair it with status text that says what is running.
  let { class: className }: { class?: string } = $props();
</script>

<span aria-hidden="true" class={cn("block h-1 overflow-hidden rounded-full bg-muted", className)}>
  <!-- Reduced motion: one pass would end off-track, so hold a static, dimmed full bar. -->
  <span
    class="block h-full w-1/3 animate-activity rounded-full bg-accent motion-reduce:w-full motion-reduce:animate-none motion-reduce:opacity-50"
  ></span>
</span>
