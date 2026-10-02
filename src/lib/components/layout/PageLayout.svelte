<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string;
    /** Right side of the header, e.g. a progress summary. */
    aside?: Snippet;
    toolbar?: Snippet;
    children: Snippet;
  }

  let { title, description, aside, toolbar, children }: Props = $props();
</script>

<div class="h-full overflow-y-auto">
  <div class="mx-auto flex max-w-275 flex-col gap-4 px-4 pt-5 pb-28 sm:px-6">
    <header class="flex flex-wrap items-end justify-between gap-x-6 gap-y-3">
      <div class="min-w-0">
        <h1 class="m-0 font-display text-[26px] leading-tight font-semibold wrap-break-word">{title}</h1>
        {#if description}
          <p class="m-0 mt-1 text-[13px] wrap-break-word text-foreground-muted">{description}</p>
        {/if}
      </div>
      {#if aside}{@render aside()}{/if}
    </header>
    {#if toolbar}
      <div class="flex flex-wrap items-center gap-2">{@render toolbar()}</div>
    {/if}
    {@render children()}
  </div>
</div>
