<script lang="ts" module>
  import { HEADING } from "$lib/design";

  const TITLE = {
    md: HEADING.section,
    lg: HEADING.pane,
    xl: HEADING.dialog,
    hero: HEADING.hero,
  };

  export type ModalTitleSize = keyof typeof TITLE;
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import { getModalTitleId } from "./modalContext";

  interface Props {
    /** `md` for confirmations, `lg` for content dialogs, `xl` for detail views, `hero` for About. */
    size?: ModalTitleSize | undefined;
    class?: string;
    children: Snippet;
  }

  let { size = "md", class: className, children }: Props = $props();

  // The id the enclosing Modal is labelled by.
  const id = getModalTitleId();
</script>

<h2 {id} class={cn("m-0 wrap-break-word text-foreground", TITLE[size], className)}>{@render children()}</h2>
