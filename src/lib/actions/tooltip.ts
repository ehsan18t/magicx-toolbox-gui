export function tooltip(node: HTMLElement, text: string | undefined | null) {
  let tooltipComponent: HTMLElement | null = null;

  function hide() {
    if (tooltipComponent) {
      tooltipComponent.remove();
      tooltipComponent = null;
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

    // Ensure we never leave an orphaned tooltip behind
    hide();

    // Create tooltip
    tooltipComponent = document.createElement("div");
    tooltipComponent.textContent = text;

    // Style tooltip
    tooltipComponent.className =
      "fixed z-[9999] px-2.5 py-1.5 text-xs font-medium text-foreground bg-elevated rounded-md shadow-lg border border-border pointer-events-none fade-in-0 zoom-in-95 animate-in duration-150";

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
