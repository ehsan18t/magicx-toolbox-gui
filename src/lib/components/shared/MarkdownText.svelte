<script lang="ts">
  /**
   * Simple markdown renderer for tweak info sections.
   * Supports: **bold**, *italic*, `code`, headers (##), bullet lists (-), numbered lists (1.), and line breaks.
   */
  interface Props {
    content: string;
    class?: string;
  }

  let { content, class: className = "" }: Props = $props();

  /**
   * Parse markdown content into HTML segments
   */
  function parseMarkdown(text: string): string {
    if (!text) return "";

    // Escape HTML entities first
    let html = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

    // Process line by line for block elements
    const lines = html.split("\n");
    const processedLines: string[] = [];
    let inList = false;
    let listType: "ul" | "ol" | null = null;

    for (let i = 0; i < lines.length; i++) {
      let line = lines[i];

      // Headers (## Header)
      if (line.startsWith("### ")) {
        if (inList) {
          processedLines.push(listType === "ul" ? "</ul>" : "</ol>");
          inList = false;
          listType = null;
        }
        processedLines.push(
          `<h4 class="mt-3 mb-1.5 text-sm font-semibold text-foreground">${processInline(line.slice(4))}</h4>`,
        );
        continue;
      }
      if (line.startsWith("## ")) {
        if (inList) {
          processedLines.push(listType === "ul" ? "</ul>" : "</ol>");
          inList = false;
          listType = null;
        }
        processedLines.push(
          `<h3 class="mt-3 mb-1.5 text-sm font-semibold text-foreground">${processInline(line.slice(3))}</h3>`,
        );
        continue;
      }

      // Bullet lists (- item or * item)
      const bulletMatch = line.match(/^[-*]\s+(.+)$/);
      if (bulletMatch) {
        if (!inList || listType !== "ul") {
          if (inList) processedLines.push(listType === "ul" ? "</ul>" : "</ol>");
          processedLines.push('<ul class="my-1.5 ml-4 list-disc space-y-0.5">');
          inList = true;
          listType = "ul";
        }
        processedLines.push(`<li class="text-foreground-muted">${processInline(bulletMatch[1])}</li>`);
        continue;
      }

      // Numbered lists (1. item)
      const numberedMatch = line.match(/^\d+\.\s+(.+)$/);
      if (numberedMatch) {
        if (!inList || listType !== "ol") {
          if (inList) processedLines.push(listType === "ul" ? "</ul>" : "</ol>");
          processedLines.push('<ol class="my-1.5 ml-4 list-decimal space-y-0.5">');
          inList = true;
          listType = "ol";
        }
        processedLines.push(`<li class="text-foreground-muted">${processInline(numberedMatch[1])}</li>`);
        continue;
      }

      // Close list if we hit a non-list line
      if (inList && line.trim() !== "") {
        processedLines.push(listType === "ul" ? "</ul>" : "</ol>");
        inList = false;
        listType = null;
      }

      // Empty lines
      if (line.trim() === "") {
        if (!inList) {
          processedLines.push('<div class="h-2"></div>');
        }
        continue;
      }

      // Regular paragraph
      processedLines.push(`<p class="text-foreground-muted">${processInline(line)}</p>`);
    }

    // Close any open list
    if (inList) {
      processedLines.push(listType === "ul" ? "</ul>" : "</ol>");
    }

    return processedLines.join("");
  }

  /**
   * Process inline markdown: **bold**, *italic*, `code`
   */
  function processInline(text: string): string {
    return (
      text
        // Bold: **text** or __text__
        .replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold text-foreground">$1</strong>')
        .replace(/__(.+?)__/g, '<strong class="font-semibold text-foreground">$1</strong>')
        // Italic: *text* or _text_
        .replace(/\*(.+?)\*/g, "<em>$1</em>")
        .replace(/_(.+?)_/g, "<em>$1</em>")
        // Inline code: `code`
        .replace(
          /`(.+?)`/g,
          '<code class="rounded bg-surface-alt px-1.5 py-0.5 font-mono text-xs text-accent">$1</code>',
        )
    );
  }

  const renderedHtml = $derived(parseMarkdown(content));
</script>

<!-- eslint-disable svelte/no-at-html-tags -- Intentional for markdown rendering, content is escaped -->
<div class="markdown-text text-sm leading-relaxed {className}">
  {@html renderedHtml}
</div>
