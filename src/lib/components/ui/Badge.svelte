<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import type { Snippet } from "svelte";
  import { badge, type BadgeVariants } from "./variants";

  interface Props extends BadgeVariants {
    tooltip?: string;
    class?: string;
    children: Snippet;
  }

  let { tone, size, case: letterCase, solid, tooltip, class: className, children }: Props = $props();
</script>

<span class={badge({ tone, size, case: letterCase, solid, class: className })} use:tooltipAction={tooltip}>
  {@render children()}
</span>
