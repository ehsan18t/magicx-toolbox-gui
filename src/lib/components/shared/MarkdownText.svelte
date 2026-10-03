<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import { openExternal } from "$lib/utils/externalLink";
  import { markdownToHtml } from "$lib/utils/markdown";
  import type { Attachment } from "svelte/attachments";
  import { on } from "svelte/events";

  interface Props {
    content: string;
    class?: string;
  }

  let { content, class: className }: Props = $props();

  const html = $derived(markdownToHtml(content));

  // {@html} anchors cannot be ExternalLinks, so one delegated listener handles them all.
  const externalLinks: Attachment<HTMLElement> = (node) =>
    on(node, "click", (event) => {
      if (event.target instanceof Element) openExternal(event, event.target.closest("a[href]")?.getAttribute("href"));
    });
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -- markdownToHtml escapes the source and allows only http(s) links. -->
<div class={cn("text-sm leading-relaxed", className)} {@attach externalLinks}>{@html html}</div>
