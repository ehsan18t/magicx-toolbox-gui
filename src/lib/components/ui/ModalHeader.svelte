<script lang="ts">
  import type { Snippet } from "svelte";
  import IconButton from "./IconButton.svelte";

  interface Props {
    /** Adds a close button after the content. */
    onclose?: () => void;
    children: Snippet;
  }

  let { onclose, children }: Props = $props();
</script>

<div class="flex shrink-0 items-start justify-between gap-3 px-6 pt-5 pb-3">
  {@render children()}
  {#if onclose}<IconButton icon="mdi:close" label="Close" onclick={onclose} />{/if}
</div>
