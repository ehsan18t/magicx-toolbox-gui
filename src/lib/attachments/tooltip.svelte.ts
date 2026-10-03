import type { Attachment } from "svelte/attachments";
import { type FlyoutPlacement, placeFlyout } from "$lib/utils/flyout";
import { delay } from "$lib/utils/motion";

const PLACEMENT: FlyoutPlacement = { side: "above", align: "center", gap: 8, inset: 4 };

type TooltipValue = string | undefined | null;
/** A function is read when the tooltip opens, e.g. to show a label only while it is cut short. */
type TooltipText = TooltipValue | ((node: HTMLElement) => TooltipValue);

/** `text` only while `target` (the anchor by default) is cut short, e.g. a truncated label. */
export const textIfCut =
  (text: string, target: (node: HTMLElement) => Element | null = (node) => node) =>
  (node: HTMLElement): string | null => {
    const el = target(node);
    return el && el.scrollWidth > el.clientWidth ? text : null;
  };

// Moving from one tooltip to the next within the delay skips it, as Windows does.
let lastHiddenAt = -Infinity;
let nextId = 0;

/** Takes a getter: a changed text updates the open tooltip in place instead of re-running the attachment. */
export const tooltip =
  (getText: () => TooltipText): Attachment<HTMLElement> =>
  (node) => {
    let text: TooltipText;
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

    // Capture phase, stopped: the first Escape dismisses the tooltip, not the dialog or field under it.
    function hideOnEscape(e: KeyboardEvent) {
      if (e.key !== "Escape" || !tip) return;
      e.preventDefault();
      e.stopPropagation();
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
        "fixed z-popover px-2.5 py-1.5 text-xs font-medium text-foreground bg-elevated rounded-md shadow-flyout border border-border pointer-events-none animate-pop-in";
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
      // Offset size, not the bounding rect: the entrance pop scales the tooltip.
      const size = { width: tip.offsetWidth, height: tip.offsetHeight };
      const viewport = { width: window.innerWidth, height: window.innerHeight };
      const { top, left } = placeFlyout(node.getBoundingClientRect(), size, viewport, PLACEMENT);
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

    $effect(() => {
      text = getText();
      const content = read();
      if (!content) hide();
      else if (tip) tip.textContent = content;
    });

    return () => {
      node.removeEventListener("mouseenter", scheduleShow);
      node.removeEventListener("mouseleave", hide);
      node.removeEventListener("mousemove", position);
      node.removeEventListener("pointerdown", hide);
      node.removeEventListener("click", hide);
      node.removeEventListener("focusin", showOnFocus);
      node.removeEventListener("focusout", hideOnFocusOut);
      hide();
    };
  };
