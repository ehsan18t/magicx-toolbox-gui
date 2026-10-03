<script lang="ts">
  import { Callout, InlineCode } from "$lib/components/ui";
  import { PROFILE_EXT } from "$lib/config/app";
  import type { Snippet } from "svelte";

  /** Completes the sentence "A .mgx profile …". */
  let { children }: { children: Snippet } = $props();
</script>

<Callout density="panel" icon="mdi:information">
  <p class="m-0 text-xs leading-relaxed text-foreground-muted">
    A <InlineCode>.{PROFILE_EXT}</InlineCode> profile {@render children()}
  </p>
</Callout>
