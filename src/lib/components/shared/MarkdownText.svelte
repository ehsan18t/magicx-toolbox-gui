<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { Attachment } from "svelte/attachments";

  /**
   * Simple markdown renderer for tweak info sections.
   * Supports: **bold**, *italic*, `code`, [links](https://…), headers (##/###),
   * bullet lists (-), numbered lists (1.), and line breaks.
   *
   * Links open in the system browser via the Tauri opener. Because the body is
   * rendered with {@html}, a Svelte component can't be embedded per-anchor, so a
   * single delegated click handler (the `externalLinks` action) intercepts clicks
   * on any rendered <a> — the same behavior as the ExternalLink component.
   */
  interface Props {
    content: string;
    class?: string;
  }

  let { content, class: className = "" }: Props = $props();

  /** Open real <a href> targets in the system browser instead of navigating the webview. */
  const externalLinks: Attachment<HTMLElement> = (node) => {
    const handler = async (event: MouseEvent) => {
      const anchor = (event.target as HTMLElement | null)?.closest?.("a[href]");
      const href = anchor?.getAttribute("href") ?? "";
      if (href.startsWith("http://") || href.startsWith("https://")) {
        event.preventDefault();
        try {
          await openUrl(href);
        } catch (error) {
          console.error(`Failed to open external link: ${href}`, error);
        }
      }
    };
    node.addEventListener("click", handler);
    return () => node.removeEventListener("click", handler);
  };

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
   * Process inline markdown: [links](url), **bold**, *italic*, `code`.
   * Links are extracted first and shielded behind a sentinel so the emphasis
   * passes below can never mangle a URL (an underscore inside a href must not
   * become <em>). The link label still receives full inline formatting.
   */
  function processInline(text: string): string {
    const links: string[] = [];
    const shielded = text.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_m, label: string, url: string) => {
      const trimmed = url.trim();
      // Only http(s) may become a link. Any other scheme (javascript:, data:, vbscript:…)
      // is dropped to plain text: rendering it as an <a href> would let a click fall
      // through to native webview navigation and execute. Defense-in-depth even though
      // the corpus is trusted, because this is a reusable shared component.
      if (!/^https?:\/\//i.test(trimmed)) return label;
      // The upstream pass already escaped & < >; quotes are the only attribute-breakout
      // risk left, so percent-encode them. Label keeps full inline formatting.
      const safeUrl = trimmed.replace(/"/g, "%22").replace(/'/g, "%27");
      const idx =
        links.push(
          `<a href="${safeUrl}" rel="noopener noreferrer" class="cursor-pointer font-medium text-accent underline decoration-accent/40 underline-offset-2 transition-colors hover:decoration-accent">${emphasis(label)}</a>`,
        ) - 1;
      // Sentinel is emphasis-inert (no * _ backtick) and cannot occur in authored
      // text, so a bare number in prose is never mistaken for a placeholder.
      return `@@LINK${idx}@@`;
    });
    return emphasis(shielded).replace(/@@LINK(\d+)@@/g, (_m, i: string) => links[Number(i)]);
  }

  /** Bold / italic / inline-code, applied to link-free text or a link's label. */
  function emphasis(text: string): string {
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
<div class="markdown-text text-sm leading-relaxed {className}" {@attach externalLinks}>
  {@html renderedHtml}
</div>
