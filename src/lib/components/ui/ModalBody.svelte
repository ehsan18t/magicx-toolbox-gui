<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
    scrollable?: boolean;
    maxHeight?: string;
  }

  const {
    children,
    class: className = "",
    scrollable = false,
    maxHeight = "calc(100dvh - 2.5rem - 6rem)",
  }: Props = $props();
</script>

<div
  class="px-5 py-5 {scrollable ? 'overflow-y-auto' : ''} {className}"
  style={scrollable ? `max-height: ${maxHeight}` : ""}
>
  {@render children()}
</div>
