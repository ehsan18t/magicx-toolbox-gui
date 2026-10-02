import { delay } from "$lib/utils/motion";

/** Gap between the anchor and the tooltip, px. */
const TOOLTIP_OFFSET = 8;
/** Closest the tooltip comes to the viewport's side edges, px. */
const VIEWPORT_INSET = 4;

type TooltipText = string | undefined | null;

// Moving from one tooltip to the next within the delay skips it, as Windows does.
let lastHiddenAt = -Infinity;

export function tooltip(node: HTMLElement, text: TooltipText) {
  let tip: HTMLElement | null = null;
  let showTimer: ReturnType<typeof setTimeout> | undefined;

  function hide() {
    clearTimeout(showTimer);
    if (tip) {
      tip.remove();
      tip = null;
      lastHiddenAt = performance.now();
    }
    window.removeEventListener("scroll", hide, true);
    window.removeEventListener("resize", position);
    document.removeEventListener("visibilitychange", hideIfHidden);
  }

  function hideIfHidden() {
    if (document.hidden) hide();
  }

  function scheduleShow() {
    if (!text) return;
    hide();
    const wait = delay("tooltip");
    if (performance.now() - lastHiddenAt < wait) show();
    else showTimer = setTimeout(show, wait);
  }

  function show() {
    if (!text) return;
    tip = document.createElement("div");
    tip.textContent = text;
    tip.className =
      "fixed z-popover px-2.5 py-1.5 text-xs font-medium text-foreground bg-elevated rounded-md shadow-lg border border-border pointer-events-none animate-pop-in";
    document.body.appendChild(tip);
    position();

    // These interrupt a hover without firing mouseleave.
    window.addEventListener("scroll", hide, true);
    window.addEventListener("resize", position);
    document.addEventListener("visibilitychange", hideIfHidden);
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
  node.addEventListener("blur", hide, true);

  return {
    update(newText: TooltipText) {
      text = newText;
      if (!text) hide();
      else if (tip) tip.textContent = text;
    },
    destroy() {
      node.removeEventListener("mouseenter", scheduleShow);
      node.removeEventListener("mouseleave", hide);
      node.removeEventListener("mousemove", position);
      node.removeEventListener("pointerdown", hide);
      node.removeEventListener("click", hide);
      node.removeEventListener("blur", hide, true);
      hide();
    },
  };
}
