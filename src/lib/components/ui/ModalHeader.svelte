<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
    /** ID for aria-labelledby connection with Modal */
    id?: string;
  }

  let { children, class: className = "", id }: Props = $props();
</script>

<div {id} class="flex items-start justify-between gap-3 border-b border-border px-5 py-4 {className}">
  {@render children()}
</div>
