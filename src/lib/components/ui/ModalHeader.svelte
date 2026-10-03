<script lang="ts" module>
  const TITLE = {
    md: "m-0 text-base font-semibold text-foreground",
    lg: "m-0 font-display text-lg font-semibold wrap-break-word text-foreground",
  };
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import IconButton from "./IconButton.svelte";
  import { getModalTitleId } from "./modalContext";

  interface Props {
    title: string;
    /** `md` for confirmations, `lg` for content dialogs. */
    size?: keyof typeof TITLE;
    /** Before the title, e.g. an icon. */
    leading?: Snippet;
    /** Before the close button, e.g. a step indicator. */
    actions?: Snippet;
    /** Adds a close button. */
    onclose?: () => void;
    class?: string;
    /** Under the title, e.g. a subtitle. */
    children?: Snippet;
  }

  let { title, size = "md", leading, actions, onclose, class: className, children }: Props = $props();

  const titleId = getModalTitleId();
</script>

<div class={cn("flex shrink-0 items-start justify-between gap-3 px-6 pt-5 pb-3", className)}>
  <div class="flex min-w-0 items-center gap-3">
    {@render leading?.()}
    <div class="min-w-0">
      <h2 id={titleId} class={TITLE[size]}>{title}</h2>
      {@render children?.()}
    </div>
  </div>
  {#if actions || onclose}
    <div class="flex shrink-0 items-center gap-2">
      {@render actions?.()}
      {#if onclose}<IconButton icon="mdi:close" label="Close" onclick={onclose} />{/if}
    </div>
  {/if}
</div>
