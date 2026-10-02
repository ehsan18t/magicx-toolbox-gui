/** Reports whether a scroll box hides content above or below its visible part. */
export function overflowHints(node: HTMLElement, onchange: (above: boolean, below: boolean) => void) {
  const update = () => {
    const above = node.scrollTop > 1;
    const below = node.scrollTop + node.clientHeight < node.scrollHeight - 1;
    onchange(above, below);
  };
  const resize = new ResizeObserver(update);
  resize.observe(node);
  // Content can grow without the box resizing: categories load, items appear.
  const mutate = new MutationObserver(update);
  mutate.observe(node, { childList: true, subtree: true });
  node.addEventListener("scroll", update, { passive: true });
  update();

  return {
    destroy() {
      resize.disconnect();
      mutate.disconnect();
      node.removeEventListener("scroll", update);
    },
  };
}
