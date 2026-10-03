import { HEADING, INLINE_CODE, PROSE_LINK_UNDERLINE } from "$lib/design";
type ListType = "ul" | "ol";

const LIST_OPEN: Record<ListType, string> = {
  ul: '<ul class="my-1.5 ml-4 list-disc space-y-0.5">',
  ol: '<ol class="my-1.5 ml-4 list-decimal space-y-0.5">',
};
const BULLET = /^[-*]\s+(.+)$/;
const NUMBERED = /^\d+\.\s+(.+)$/;
const LINK = /\[([^\]]+)\]\(([^)]+)\)/g;
// Only http(s) becomes a link: another scheme (javascript:, data:) would run on a click the webview handles.
const SAFE_URL = /^https?:\/\//i;
// Emphasis-inert (no * _ or backtick), so the emphasis pass cannot mangle a shielded URL.
const LINK_SENTINEL = /@@LINK(\d+)@@/g;
const STRONG = '<strong class="font-semibold text-foreground">$1</strong>';

const escapeHtml = (text: string) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function emphasis(text: string): string {
  return text
    .replace(/\*\*(.+?)\*\*/g, STRONG)
    .replace(/__(.+?)__/g, STRONG)
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/_(.+?)_/g, "<em>$1</em>")
    .replace(/`(.+?)`/g, `<code class="${INLINE_CODE}">$1</code>`);
}

function inline(text: string): string {
  const links: string[] = [];
  const shielded = text.replace(LINK, (_match, label: string, url: string) => {
    const trimmed = url.trim();
    if (!SAFE_URL.test(trimmed)) return label;
    // & < > are already escaped; quotes are the only attribute breakout left.
    const href = trimmed.replace(/"/g, "%22").replace(/'/g, "%27");
    links.push(
      `<a href="${href}" rel="noopener noreferrer" class="cursor-pointer font-medium text-accent underline ${PROSE_LINK_UNDERLINE} underline-offset-2 transition-colors hover:decoration-accent">${emphasis(label)}</a>`,
    );
    return `@@LINK${links.length - 1}@@`;
  });
  return emphasis(shielded).replace(LINK_SENTINEL, (_match, i: string) => links[Number(i)]);
}

/** HTML-escaped first. Headings (## ###), lists (- * 1.), **bold**, *italic*, `code`, [links](https://…). */
export function markdownToHtml(text: string): string {
  const out: string[] = [];
  let list: ListType | null = null;

  const closeList = () => {
    if (list) out.push(`</${list}>`);
    list = null;
  };
  const listItem = (type: ListType, content: string) => {
    if (list !== type) {
      closeList();
      out.push(LIST_OPEN[type]);
      list = type;
    }
    out.push(`<li class="text-foreground-muted">${inline(content)}</li>`);
  };
  const heading = (tag: "h3" | "h4", content: string) => {
    closeList();
    out.push(`<${tag} class="mt-3 mb-1.5 ${HEADING.item} text-foreground">${inline(content)}</${tag}>`);
  };

  for (const line of text ? escapeHtml(text).split("\n") : []) {
    if (line.startsWith("### ")) heading("h4", line.slice(4));
    else if (line.startsWith("## ")) heading("h3", line.slice(3));
    else if (BULLET.test(line)) listItem("ul", line.replace(BULLET, "$1"));
    else if (NUMBERED.test(line)) listItem("ol", line.replace(NUMBERED, "$1"));
    // A blank line inside a list keeps it open.
    else if (line.trim() === "") {
      if (!list) out.push('<div class="h-2"></div>');
    } else {
      closeList();
      out.push(`<p class="text-foreground-muted">${inline(line)}</p>`);
    }
  }
  closeList();
  return out.join("");
}
