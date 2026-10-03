import { delay } from "$lib/utils/motion";

/** Gap between the anchor and the tooltip, px. */
const TOOLTIP_OFFSET = 8;
/** Closest the tooltip comes to the viewport's side edges, px. */
const VIEWPORT_INSET = 4;

type TooltipValue = string | undefined | null;
/** A function is read when the tooltip opens, e.g. to show a label only while it is cut short. */
type TooltipText = TooltipValue | ((node: HTMLElement) => TooltipValue);

// Moving from one tooltip to the next within the delay skips it, as Windows does.
let lastHiddenAt = -Infinity;
let nextId = 0;

export function tooltip(node: HTMLElement, text: TooltipText) {
  let tip: HTMLElement | null = null;
  let showTimer: ReturnType<typeof setTimeout> | undefined;
  const id = `tooltip-${nextId++}`;

  const read = () => (typeof text === "function" ? text(node) : text);

  function hide() {
    clearTimeout(showTimer);
    if (tip) {
      tip.remove();
      tip = null;
      lastHiddenAt = performance.now();
      if (node.getAttribute("aria-describedby") === id) node.removeAttribute("aria-describedby");
    }
    window.removeEventListener("scroll", hide, true);
    window.removeEventListener("resize", position);
    document.removeEventListener("visibilitychange", hideIfHidden);
    document.removeEventListener("keydown", hideOnEscape, true);
  }

  // Capture phase: the first Escape dismisses the tooltip, not the dialog under it.
  function hideOnEscape(e: KeyboardEvent) {
    if (e.key !== "Escape" || !tip) return;
    e.preventDefault();
    hide();
  }

  function showOnFocus() {
    if (node.matches(":focus-visible") || node.querySelector(":focus-visible")) scheduleShow();
  }

  function hideOnFocusOut(e: FocusEvent) {
    if (!(e.relatedTarget instanceof Node && node.contains(e.relatedTarget))) hide();
  }

  function hideIfHidden() {
    if (document.hidden) hide();
  }

  function scheduleShow() {
    if (!read()) return;
    hide();
    const wait = delay("tooltip");
    if (performance.now() - lastHiddenAt < wait) show();
    else showTimer = setTimeout(show, wait);
  }

  function show() {
    const content = read();
    if (!content) return;
    tip = document.createElement("div");
    tip.id = id;
    tip.setAttribute("role", "tooltip");
    tip.textContent = content;
    tip.className =
      "fixed z-popover px-2.5 py-1.5 text-xs font-medium text-foreground bg-elevated rounded-md shadow-lg border border-border pointer-events-none animate-pop-in";
    document.body.appendChild(tip);
    position();
    // Describes only when it adds to the name; a tooltip that repeats the label stays out of the way.
    const name = node.getAttribute("aria-label") ?? node.textContent?.trim();
    if (content !== name && !node.hasAttribute("aria-describedby")) node.setAttribute("aria-describedby", id);

    // These interrupt a hover without firing mouseleave.
    window.addEventListener("scroll", hide, true);
    window.addEventListener("resize", position);
    document.addEventListener("visibilitychange", hideIfHidden);
    document.addEventListener("keydown", hideOnEscape, true);
  }

  function position() {
    if (!tip) return;
    const anchor = node.getBoundingClientRect();
    // Offset size, not the bounding rect: the entrance pop scales the tooltip.
    const width = tip.offsetWidth;
    const height = tip.offsetHeight;

    // Centred above, flipped below when there is no room.
    let top = anchor.top - height - TOOLTIP_OFFSET;
    if (top < 0) top = anchor.bottom + TOOLTIP_OFFSET;
    let left = anchor.left + (anchor.width - width) / 2;
    if (left < 0) left = VIEWPORT_INSET;
    if (left + width > window.innerWidth) left = window.innerWidth - width - VIEWPORT_INSET;

    tip.style.top = `${top}px`;
    tip.style.left = `${left}px`;
  }

  node.addEventListener("mouseenter", scheduleShow);
  node.addEventListener("mouseleave", hide);
  node.addEventListener("mousemove", position);
  node.addEventListener("pointerdown", hide);
  node.addEventListener("click", hide);
  node.addEventListener("focusin", showOnFocus);
  node.addEventListener("focusout", hideOnFocusOut);

  return {
    update(newText: TooltipText) {
      text = newText;
      const content = read();
      if (!content) hide();
      else if (tip) tip.textContent = content;
    },
    destroy() {
      node.removeEventListener("mouseenter", scheduleShow);
      node.removeEventListener("mouseleave", hide);
      node.removeEventListener("mousemove", position);
      node.removeEventListener("pointerdown", hide);
      node.removeEventListener("click", hide);
      node.removeEventListener("focusin", showOnFocus);
      node.removeEventListener("focusout", hideOnFocusOut);
      hide();
    },
  };
}
