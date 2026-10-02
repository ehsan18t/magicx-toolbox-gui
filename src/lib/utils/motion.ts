// Motion presets. Timing lives once, as tokens in app.css; this reads them so JS and CSS motion match.
// CSS classes (`animate-*`, `duration-*`, `ease-*`) for enter-only and hover motion; these presets where
// Svelte must hold an element for its exit or measure its height.
import { flip } from "svelte/animate";
import type { AnimationConfig } from "svelte/animate";
import { slide, type EasingFunction, type TransitionConfig } from "svelte/transition";

type Speed = "fast" | "normal" | "slow" | "slower" | "highlight";
type Delay = "reveal" | "settle" | "tooltip" | "feedback";
type Curve = "in" | "out" | "in-out" | "overshoot";
type Distance = "md" | "lg";
type Direction = "above" | "below" | "left" | "right";

interface MotionParams {
  speed?: Speed;
}

const tokens = new Map<string, string>();

function token(name: string): string {
  let value = tokens.get(name);
  if (value === undefined) {
    value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (value) tokens.set(name, value);
  }
  return value;
}

// WAAPI-driven Svelte transitions ignore the CSS reduced-motion override, so each preset checks it.
export function reducedMotion(): boolean {
  return matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function ms(name: string): number {
  const value = token(name);
  return value.endsWith("ms") ? parseFloat(value) : parseFloat(value) * 1000;
}

export function duration(speed: Speed): number {
  return reducedMotion() ? 0 : ms(`--transition-duration-${speed}`);
}

/** Intent and anti-flicker waits, so not zeroed by reduced motion. */
export function delay(name: Delay): number {
  return ms(`--transition-delay-${name}`);
}

function distance(size: Distance): number {
  return parseFloat(token(`--motion-distance-${size}`));
}

function cubicBezier(x1: number, y1: number, x2: number, y2: number): EasingFunction {
  const at = (a: number, b: number, t: number) => 3 * a * t * (1 - t) ** 2 + 3 * b * t ** 2 * (1 - t) + t ** 3;
  return (x) => {
    if (x <= 0 || x >= 1) return x <= 0 ? 0 : 1;
    let lo = 0;
    let hi = 1;
    for (let i = 0; i < 20; i++) {
      const mid = (lo + hi) / 2;
      if (at(x1, x2, mid) < x) lo = mid;
      else hi = mid;
    }
    return at(y1, y2, (lo + hi) / 2);
  };
}

const curves = new Map<Curve, EasingFunction>();

function easing(curve: Curve): EasingFunction {
  let fn = curves.get(curve);
  if (!fn) {
    const [x1, y1, x2, y2] = (token(`--ease-${curve}`).match(/-?[\d.]+/g) ?? ["0", "0", "1", "1"]).map(Number);
    fn = cubicBezier(x1, y1, x2, y2);
    curves.set(curve, fn);
  }
  return fn;
}

export function fade(_node: Element, { speed = "normal" }: MotionParams = {}): TransitionConfig {
  return { duration: duration(speed), easing: easing("out"), css: (t) => `opacity: ${t}` };
}

/** Fades in while travelling from `from` toward its place. */
export function shift(
  _node: Element,
  { speed = "slow", from = "below", by = "md" }: MotionParams & { from?: Direction; by?: Distance } = {},
): TransitionConfig {
  const offset = distance(by) * (from === "below" || from === "right" ? 1 : -1);
  const axis = from === "left" || from === "right" ? "X" : "Y";
  return {
    duration: duration(speed),
    easing: easing("out"),
    // `translate`/`scale`, not `transform`: Svelte's `animate:` keeps a leaving item in place through `transform`.
    css: (t, u) => `opacity: ${t}; translate: ${axis === "X" ? `${u * offset}px 0` : `0 ${u * offset}px`}`,
  };
}

/** Flyouts and popovers: grow from slightly smaller; set `transform-origin` on the node. */
export function pop(_node: Element, { speed = "normal" }: MotionParams = {}): TransitionConfig {
  const from = 1 - parseFloat(token("--motion-scale-delta"));
  return {
    duration: duration(speed),
    easing: easing("out"),
    css: (t) => `opacity: ${t}; scale: ${from + (1 - from) * t}`,
  };
}

/** Height (or width) reveal with a fade, for inline panels that push content. */
export function expand(
  node: Element,
  { speed = "normal", axis = "y" }: MotionParams & { axis?: "x" | "y" } = {},
): TransitionConfig {
  const base = slide(node, { duration: duration(speed), easing: easing("out"), axis });
  return { ...base, css: (t, u) => `${base.css?.(t, u) ?? ""}; opacity: ${t}` };
}

/** Slides `to`'s pseudo-element (a selection pill) in from where `from`'s sat; only the control that changed pays. */
export function glide(from: HTMLElement, to: HTMLElement, pseudoElement = "::before"): void {
  const time = duration("normal");
  if (!time) return;
  const offset = from.offsetLeft - to.offsetLeft;
  to.animate(
    { transform: [`translateX(${offset}px) scaleX(${from.offsetWidth / to.offsetWidth})`, "none"] },
    { duration: time, easing: token("--ease-out"), pseudoElement },
  );
}

/** `animate:` for keyed lists whose siblings move when one is added or removed. */
export function reflow(node: Element, rects: { from: DOMRect; to: DOMRect }): AnimationConfig {
  return flip(node, rects, { duration: duration("normal"), easing: easing("out") });
}
