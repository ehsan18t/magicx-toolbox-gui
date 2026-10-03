export interface FlyoutBox {
  top: number;
  bottom: number;
  left: number;
  width: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface FlyoutPlacement {
  /** Preferred side; flips to the other when it overflows and the other side fits. */
  side: "above" | "below";
  align: "center" | "start";
  gap: number;
  /** Closest the flyout comes to the viewport's edges, px. */
  inset: number;
}

/** Viewport position of a fixed flyout of `size` next to `anchor`, clamped inside `viewport`. */
export function placeFlyout(
  anchor: FlyoutBox,
  size: Size,
  viewport: Size,
  { side, align, gap, inset }: FlyoutPlacement,
): { top: number; left: number; above: boolean } {
  const aboveTop = anchor.top - size.height - gap;
  const belowTop = anchor.bottom + gap;
  const fitsAbove = aboveTop >= inset;
  const fitsBelow = belowTop + size.height <= viewport.height - inset;
  const above = side === "above" ? fitsAbove || !fitsBelow : !fitsBelow && fitsAbove;

  const preferredLeft = align === "center" ? anchor.left + (anchor.width - size.width) / 2 : anchor.left;
  const left = Math.max(inset, Math.min(preferredLeft, viewport.width - inset - size.width));
  return { top: above ? aboveTop : belowTop, left, above };
}
