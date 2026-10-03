<script lang="ts">
  import { HEADING } from "$lib/design";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string | undefined;
    /** Reads description changes out, e.g. a result count. */
    announce?: boolean;
    /** A row under the title: the page's stats, then its actions. */
    aside?: Snippet | undefined;
    children: Snippet;
  }

  let { title, description, announce = false, aside, children }: Props = $props();
</script>

<div class="h-full overflow-y-auto">
  <div class="mx-auto flex max-w-page flex-col gap-4 px-4 pt-5 pb-dock-clearance sm:px-6">
    <header class="flex flex-col gap-3">
      <div class="min-w-0">
        <!-- Focusable for utils/focus: where focus lands when the control holding it goes away. -->
        <h1 tabindex="-1" class={["m-0 wrap-break-word", HEADING.page]}>{title}</h1>
        {#if description}
          <p class="m-0 mt-1 text-ui wrap-break-word text-foreground-muted" aria-live={announce ? "polite" : undefined}>
            {description}
          </p>
        {/if}
      </div>
      {#if aside}
        <div class="flex flex-wrap items-center gap-x-4 gap-y-2">{@render aside()}</div>
      {/if}
    </header>
    {@render children()}
  </div>
</div>
