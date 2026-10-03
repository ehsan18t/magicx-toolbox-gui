# Roadmap

Improvements that are **researched, wanted and deferred**: not defects (those live in `KNOWN_ISSUES.md`) and not rejected ideas (those are ADRs). Each entry carries the analysis already done, where the change lands, what it gains, what can go wrong and what it depends on, so the work can start without a second audit. Entries leave this file by shipping, or by becoming an ADR if we decide against them.

File references name symbols rather than line numbers, since lines drift. The target renderer is the evergreen WebView2 (Chromium), so Chromium-only platform features are fair game; anything newer than about Chrome 133 still needs an `@supports` fallback, because LTSC machines that block updates can lag behind.

| #   | Improvement                                                          | Size | Depends on |
| --- | -------------------------------------------------------------------- | ---- | ---------- |
| 1   | Popover API and CSS anchor positioning for `Select` and tooltips     | M    |            |
| 2   | Native `<dialog>` for `Modal`, `inert` for the overlay nav drawer    | M    | 1          |
| 3   | View Transitions for page switches, theme swaps and the nav glide    | M    | 2          |
| 4   | Customizable native `<select>` (`appearance: base-select`)           | L    | 1          |
| 5   | `noUncheckedIndexedAccess`                                           | M    |            |
| 6   | Small polish: row `content-visibility`, `field-sizing`, rune helpers | S    |            |
| 7   | Product decisions held back from the modernization pass              | S    |            |

The order of 1 to 3 is forced: a native modal dialog renders in the top layer, above every `z-index`, so toasts, tooltips and the applying overlay must reach the top layer (as popovers) before `Modal` moves there, or they would render under an open dialog and toast buttons would turn inert.

---

## 1. Popover API and CSS anchor positioning for `Select` and tooltips

**Today.** `src/lib/utils/flyout.ts` (`placeFlyout`, with its test) measures with `getBoundingClientRect` and positions menus in JS. `Select.svelte` calls it from `updatePosition` and keeps its own outside-click, resize and scroll listeners, closing the menu when the page scrolls. The tooltip attachment (`src/lib/attachments/tooltip.svelte.ts`, `position`) repositions on every `mousemove`. Stacking is managed with the `z-popover` token.

**The change.** Menus and tooltips become `popover` elements (`popover="auto"` for the Select menu, `popover="hint"` for tooltips, Chrome 133) positioned with `anchor-name` / `position-anchor`, `position-area`, `position-try-fallbacks: flip-block`, `min-width: anchor-size(width)` and `justify-self: anchor-center` (anchor positioning since Chrome 125, `position-try-fallbacks` since 128). Toasts and the applying overlay become manual popovers so they live in the top layer too (prerequisite for 2).

**Gains.** `placeFlyout` and every measurement go; the menu follows its trigger while scrolling instead of closing; click-outside and Escape dismissal come free with `popover="auto"`; the top layer replaces z-index ordering.

**Risks.** A popover closing on Escape does not `preventDefault` the keydown, so `Modal`'s window-level Escape handler would also close the dialog underneath; doing 2 removes this, because close watchers close only the topmost layer, but until then the handler must check for an open popover. The `origin-top` / `origin-bottom` flip of the `pop` motion preset needs anchored container queries (very recent Chrome); without them the pop animates from the centre. Motion must stay on the tokens and the reduced-motion override.

## 2. Native `<dialog>` for `Modal`, `inert` for the overlay nav drawer

**Today.** `Modal.svelte` hand-rolls a modal stack (`openStack`), a `FOCUSABLE` selector list, a Tab trap, a `focusin` reclaim, backdrop-click handling and a window-level Escape handler; the scrim is a separate element (`variants.ts`, the modal scrim variant). An `$effect` chain mirrors the `open` prop into `isVisible` / `isClosing`, which later effects read. `Sidebar.svelte` has its own `containTab` for the overlay drawer.

**The change.** `Modal` renders a `<dialog>` opened with `showModal()`; the scrim becomes `::backdrop`; `closedby` (`any`, `closerequest`, `none`, Chrome 134) maps onto `closeOnBackdrop` / `closeOnEscape`. The `cancel` event is routed to `onclose` with `preventDefault`, so the `open` prop stays parent-controlled (a parent may decline). `isVisible` becomes a writable `$derived` of `open`, removing the effect chain. Keep `reclaimFocus` for an opener that was removed while the dialog was open. The nav drawer docks and overlays as one element, so it stays a plain element and sets `inert` on the workspace while it overlays, as `ApplyingOverlay` already does.

**Gains.** Focus containment, an inert background (more reliable for screen readers than `aria-modal` alone), stacking and focus return become the browser's job: roughly 80 lines go from `Modal.svelte`.

**Risks.** Every one of the 16 `Modal` users needs its focus and accessibility behaviour re-checked against WCAG 2.2 AA. A CSS exit (`@starting-style`, `transition-behavior: allow-discrete` on `display` and `overlay`) is reached by the reduced-motion override, but conflicts with the CLAUDE.md rule that exit motion uses the `motion` presets; the current exit is already a CSS animation, so amend the rule in the same change.

## 3. View Transitions for page switches, theme swaps and the nav glide

**Today.** On a page switch (`Workspace.svelte`, the keyed `{#key activeTab}` block) the new page rises in while the old one vanishes at once. The theme swap (`theme.svelte.ts`, `set`) adds a `theme-transitioning` class that freezes transitions and paints an `::after` overlay (`app.css`), with an exemption for `.theme-toggle`. The sidebar active indicator glides using offsets measured by hand in `Sidebar.svelte`.

**The change.** `document.startViewTransition` around the state change (with `flushSync` so Svelte updates the DOM inside the callback); `view-transition-name` on the indicator for the glide. Durations come from the motion tokens.

**Gains.** A real crossfade from old to new; the theme-freeze hack and the measured indicator offsets go.

**Risks.** The global reduced-motion override in `app.css` does not match `::view-transition-*` pseudo-elements: extend it, or skip the transition when `reducedMotion()` is true. Clicks land on a snapshot for the duration (about 150 ms).

## 4. Customizable native `<select>` (`appearance: base-select`)

**Today.** `Select.svelte` is a hand-built listbox of about 250 lines, with its own keyboard handling built on `nextEnabledIndex`; it has no type-ahead, which the WAI-ARIA listbox pattern expects.

**The change.** A native `<select>` styled through `appearance: base-select`, `::picker(select)` and `<selectedcontent>` (Chrome 135). Either do this instead of the Select half of 1, or after 1 has proved anchor positioning out.

**Gains.** Native type-ahead, semantics, top layer and anchoring, with almost no JS.

**Risks.** The component is controlled (the parent may decline a change), while a native select commits immediately, so a declined change must be reverted. `aria-disabled` does not stop a native select opening, so opening must be blocked in the event handlers. Pending and loading styling moves into the picker pseudo-elements.

## 5. `noUncheckedIndexedAccess`

Turning it on reports about 70 `tsc` errors and about 100 `svelte-check` errors, concentrated in `src/lib/utils/fieldRanges.ts` and `src/lib/utils/changeMatrix.test.ts`. Each error is an index read that assumes presence; the fix is a guard or a non-null assertion where presence is proven. Worth doing once the code it touches is quiet, since it changes many lines without changing behaviour.

## 6. Small polish

- **`content-visibility: auto` on item rows** (`ItemRow.svelte`): category and search pages render many rows, each with a `Select` or `SegmentedSwitch`. Tailwind has no utility for it, so add an `@utility` with `contain-intrinsic-size: auto <height>`. Measure first; it pays off only at 50 rows or more.
- **`field-sizing: content` on `TextArea.svelte`** (used only by `ProfileExportModal`): the textarea grows with its content, with no JS.
- **`getAbortSignal()`** in `snapshotHistory.svelte.ts` for the effect-driven `latestRequest` guard. `discard()` must still bump the counter itself, so the gain is small.
- **`createSubscriber`** for `isMaximized` in `WindowControls.svelte`, only once a second component needs that value.
- **Status scan over a `Channel`** instead of the global `tweak-status` event (`spawn_full_scan` in `commands/tweaks.rs`, the listen-before-invoke latch in `tweaksData.svelte.ts`): removes the latch and adds an end-of-scan signal. The current code is correct; this is simplification only.

## 7. Product decisions held back from the modernization pass

- **Follow the OS theme live** until the user picks one: `theme.svelte.ts` reads `prefers-color-scheme` once at startup; `MediaQuery` from `svelte/reactivity` would make it reactive.
- **Signed updates**: the custom updater verifies the trusted URL and GitHub's SHA-256 digest, so a compromised GitHub account could still publish a build. `tauri-plugin-updater` adds a minisign signature independent of GitHub, at the cost of a key-management workflow.
- **Svelte async mode** (`$derived(await ...)`, `await` in markup, boundary `pending`): would remove the stale-response guard in `snapshotHistory`, but stays behind `experimental.async`; adopt once it is stable.
