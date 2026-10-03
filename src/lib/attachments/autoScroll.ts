import type { Attachment } from "svelte/attachments";

/** Scrolled within this of the bottom, px, the box keeps following new content. */
const FOLLOW_THRESHOLD_PX = 24;

/** Keeps a scroll box pinned to its newest content until the reader scrolls up; scrolling back down re-pins. */
export const autoScroll: Attachment<HTMLElement> = (node) => {
  let stuck = true;
  const pin = () => {
    if (stuck) node.scrollTop = node.scrollHeight;
  };
  const track = () => {
    stuck = node.scrollHeight - node.scrollTop - node.clientHeight < FOLLOW_THRESHOLD_PX;
  };
  // characterData too: a line keyed by index can be rewritten in place.
  const mutate = new MutationObserver(pin);
  mutate.observe(node, { childList: true, subtree: true, characterData: true });
  // Shrinking keeps scrollTop, so the newest line would drop below the fold.
  const resize = new ResizeObserver(pin);
  resize.observe(node);
  node.addEventListener("scroll", track, { passive: true });
  pin();

  return () => {
    mutate.disconnect();
    resize.disconnect();
    node.removeEventListener("scroll", track);
  };
};
