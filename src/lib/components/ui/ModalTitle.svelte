<script lang="ts" module>
  const TITLE = {
    md: "m-0 text-base font-semibold text-foreground",
    lg: "m-0 font-display text-lg font-semibold wrap-break-word text-foreground",
    xl: "m-0 font-display text-xl font-semibold wrap-break-word",
    hero: "m-0 font-display text-hero leading-none font-semibold tracking-display",
  };

  export type ModalTitleSize = keyof typeof TITLE;
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import { getModalTitleId } from "./modalContext";

  interface Props {
    /** `md` for confirmations, `lg` for content dialogs, `xl` for detail views, `hero` for About. */
    size?: ModalTitleSize;
    class?: string;
    children: Snippet;
  }

  let { size = "md", class: className, children }: Props = $props();

  // The id the enclosing Modal is labelled by.
  const id = getModalTitleId();
</script>

<h2 {id} class={cn(TITLE[size], className)}>{@render children()}</h2>
