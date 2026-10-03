import type { Attachment } from "svelte/attachments";

// Scroll offsets can be fractional, so the ends never compare exactly.
const SUBPIXEL = 1;

/** Reports whether a scroll box hides content above or below its visible part. */
export const overflowHints =
  (onchange: (above: boolean, below: boolean) => void): Attachment<HTMLElement> =>
  (node) => {
    const update = () => {
      const above = node.scrollTop > SUBPIXEL;
      const below = node.scrollTop + node.clientHeight < node.scrollHeight - SUBPIXEL;
      onchange(above, below);
    };
    const resize = new ResizeObserver(update);
    resize.observe(node);
    // Content can grow without the box resizing: categories load, items appear.
    const mutate = new MutationObserver(update);
    mutate.observe(node, { childList: true, subtree: true });
    node.addEventListener("scroll", update, { passive: true });
    update();

    return () => {
      resize.disconnect();
      mutate.disconnect();
      node.removeEventListener("scroll", update);
    };
  };
