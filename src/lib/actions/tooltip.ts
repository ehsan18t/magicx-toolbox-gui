export function tooltip(node: HTMLElement, text: string | undefined | null) {
  let tooltipComponent: HTMLElement | null = null;

  function params(text: string | undefined | null) {
    if (!text) return;

    // Update text if tooltip exists
    if (tooltipComponent) {
      tooltipComponent.textContent = text;
    }
  }

  function mouseEnter() {
    if (!text) return;

    // Create tooltip
    tooltipComponent = document.createElement("div");
    tooltipComponent.textContent = text;

    // Style tooltip
    tooltipComponent.className =
      "fixed z-[9999] px-2.5 py-1.5 text-xs font-medium text-foreground bg-elevated rounded-md shadow-lg border border-border pointer-events-none fade-in-0 zoom-in-95 animate-in duration-150";

    document.body.appendChild(tooltipComponent);

    positionTooltip();
  }

  function mouseLeave() {
    if (tooltipComponent) {
      tooltipComponent.remove();
      tooltipComponent = null;
    }
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

  return {
    update(newText: string) {
      text = newText;
      params(text);
    },
    destroy() {
      node.removeEventListener("mouseenter", mouseEnter);
      node.removeEventListener("mouseleave", mouseLeave);
      node.removeEventListener("mousemove", positionTooltip);
      mouseLeave();
    },
  };
}
