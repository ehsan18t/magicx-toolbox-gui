<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string;
    /** A row under the title: the page's stats, then its actions. */
    aside?: Snippet;
    children: Snippet;
  }

  let { title, description, aside, children }: Props = $props();
</script>

<div class="h-full overflow-y-auto">
  <div class="mx-auto flex max-w-page flex-col gap-4 px-4 pt-5 pb-dock-clearance sm:px-6">
    <header class="flex flex-col gap-3">
      <div class="min-w-0">
        <h1 class="m-0 font-display text-title leading-tight font-semibold wrap-break-word">{title}</h1>
        {#if description}
          <p class="m-0 mt-1 text-ui wrap-break-word text-foreground-muted">{description}</p>
        {/if}
      </div>
      {#if aside}
        <div class="flex flex-wrap items-center gap-x-4 gap-y-2">{@render aside()}</div>
      {/if}
    </header>
    {@render children()}
  </div>
</div>
