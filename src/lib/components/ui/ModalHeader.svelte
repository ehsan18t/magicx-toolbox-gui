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

<div {id} class="flex shrink-0 items-start justify-between gap-3 px-6 pt-5 pb-3 {className}">
  {@render children()}
</div>
