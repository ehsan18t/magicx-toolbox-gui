<script lang="ts">
  import type { Snippet } from "svelte";
  import { badge, type BadgeVariants } from "./variants";

  interface Props extends BadgeVariants {
    class?: string;
    children: Snippet;
  }

  let { tone, size, class: className, children }: Props = $props();
</script>

<span class={badge({ tone, size, class: className })}>
  {@render children()}
</span>
