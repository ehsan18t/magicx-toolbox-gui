<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string;
    /** `card`: padded, for a bordered list; `plain`: unpadded, between dividers. */
    variant?: "card" | "plain";
    /** The control. */
    children: Snippet;
  }

  let { title, description, variant = "card", children }: Props = $props();
</script>

{#snippet text()}
  <p class="m-0 text-ui font-medium">{title}</p>
  {#if description}<p class="m-0 mt-0.5 text-xs text-foreground-muted">{description}</p>{/if}
{/snippet}

{#if variant === "card"}
  <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-3.5">
    <div class="min-w-0 flex-1 basis-64">{@render text()}</div>
    <div class="flex shrink-0 flex-wrap items-center gap-2">{@render children()}</div>
  </div>
{:else}
  <div class="flex items-center justify-between gap-6 py-3">
    <div class="min-w-0">{@render text()}</div>
    {@render children()}
  </div>
{/if}
