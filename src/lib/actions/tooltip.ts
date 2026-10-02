import { delay } from "$lib/utils/motion";

// Moving from one tooltip to the next within the delay skips it, as Windows does.
let lastHiddenAt = -Infinity;

export function tooltip(node: HTMLElement, text: string | undefined | null) {
  let tooltipComponent: HTMLElement | null = null;
  let showTimer: ReturnType<typeof setTimeout> | undefined;

  function hide() {
    clearTimeout(showTimer);
    if (tooltipComponent) {
      tooltipComponent.remove();
      tooltipComponent = null;
      lastHiddenAt = performance.now();
    }

    window.removeEventListener("scroll", hide, true);
    window.removeEventListener("resize", positionTooltip);
    document.removeEventListener("visibilitychange", handleVisibilityChange);
  }

  function handleVisibilityChange() {
    if (document.hidden) hide();
  }

  function params(text: string | undefined | null) {
    if (!text) {
      hide();
      return;
    }

    if (tooltipComponent) tooltipComponent.textContent = text;
  }

  function mouseEnter() {
    if (!text) return;
    hide();
    const wait = delay("tooltip");
    if (performance.now() - lastHiddenAt < wait) show();
    else showTimer = setTimeout(show, wait);
  }

  function show() {
    if (!text) return;

    tooltipComponent = document.createElement("div");
    tooltipComponent.textContent = text;

    // Style tooltip
    tooltipComponent.className =
      "fixed z-popover px-2.5 py-1.5 text-xs font-medium text-foreground bg-elevated rounded-md shadow-lg border border-border pointer-events-none animate-pop-in";

    document.body.appendChild(tooltipComponent);

    positionTooltip();

    // Hide tooltip on interactions that can interrupt hover without firing mouseleave
    window.addEventListener("scroll", hide, true);
    window.addEventListener("resize", positionTooltip);
    document.addEventListener("visibilitychange", handleVisibilityChange);
  }

  function mouseLeave() {
    hide();
  }

  function positionTooltip() {
    if (!tooltipComponent) return;

    const nodeRect = node.getBoundingClientRect();
    const tooltipRect = tooltipComponent.getBoundingClientRect();

    // Position above centered
    let top = nodeRect.top - tooltipRect.height - 8;
    let left = nodeRect.left + (nodeRect.width - tooltipRect.width) / 2;

    // Boundary text (viewport) - basic check
    if (top < 0) {
      // Flip to bottom if too close to top
      top = nodeRect.bottom + 8;
    }

    if (left < 0) left = 4;
    if (left + tooltipRect.width > window.innerWidth) {
      left = window.innerWidth - tooltipRect.width - 4;
    }

    tooltipComponent.style.top = `${top}px`;
    tooltipComponent.style.left = `${left}px`;
  }

  node.addEventListener("mouseenter", mouseEnter);
  node.addEventListener("mouseleave", mouseLeave);
  node.addEventListener("mousemove", positionTooltip); // Follow/update if needed, or mostly static
  node.addEventListener("pointerdown", hide);
  node.addEventListener("click", hide);
  node.addEventListener("blur", hide, true);

  return {
    update(newText: string) {
      text = newText;
      params(text);
    },
    destroy() {
      node.removeEventListener("mouseenter", mouseEnter);
      node.removeEventListener("mouseleave", mouseLeave);
      node.removeEventListener("mousemove", positionTooltip);
      node.removeEventListener("pointerdown", hide);
      node.removeEventListener("click", hide);
      node.removeEventListener("blur", hide, true);
      hide();
    },
  };
}
