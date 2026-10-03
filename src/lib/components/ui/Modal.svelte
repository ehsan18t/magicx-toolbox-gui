<script lang="ts" module>
  // Only the topmost modal handles keys and focus: stacked focus traps pull focus back and forth forever.
  const openStack: object[] = [];

  const FOCUSABLE = [
    "a[href]",
    "summary",
    "button:not([disabled])",
    "input:not([disabled])",
    "select:not([disabled])",
    "textarea:not([disabled])",
    "[tabindex]:not([tabindex='-1'])",
  ].join(",");
</script>

<script lang="ts">
  import { focusFallback, reclaimFocus } from "$lib/utils/focus";
  import type { Snippet } from "svelte";
  import { onDestroy, tick } from "svelte";
  import { setModalTitleId } from "./modalContext";
  import { modal, type ModalSize } from "./variants";

  interface Props {
    open: boolean;
    onclose?: () => void;
    size?: ModalSize;
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    role?: "dialog" | "alertdialog";
    describedBy?: string;
    children: Snippet;
  }

  let {
    open,
    onclose,
    size = "md",
    closeOnBackdrop = true,
    closeOnEscape = true,
    role = "dialog",
    describedBy,
    children,
  }: Props = $props();

  const titleId = $props.id();
  setModalTitleId(titleId);

  // Mounted while open or playing the exit animation.
  let isVisible = $state(false);
  let isClosing = $state(false);

  let modalEl = $state<HTMLElement | null>(null);
  let previouslyFocusedEl: HTMLElement | null = null;
  // A drag that starts inside (selecting text) and ends on the scrim is not a backdrop click.
  let pressedScrim = false;

  const stackToken = {};
  const isTopmost = () => openStack.at(-1) === stackToken;

  $effect(() => {
    if (!isVisible || isClosing) return;
    openStack.push(stackToken);
    return () => void openStack.splice(openStack.indexOf(stackToken), 1);
  });

  // Keeps aria-disabled controls: they stay focusable to announce why they are blocked.
  function getFocusableElements(root: HTMLElement): HTMLElement[] {
    return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter((el) => {
      if (el.hasAttribute("disabled")) return false;
      if (el.closest("[inert]")) return false;
      return el.offsetParent !== null || el === document.activeElement;
    });
  }

  async function focusInitialElement() {
    if (!modalEl) return;
    await tick();

    const focusables = getFocusableElements(modalEl);
    (focusables.find((el) => el.getAttribute("aria-disabled") !== "true") ?? focusables[0] ?? modalEl).focus();
  }

  $effect(() => {
    if (open && !isVisible && !isClosing) {
      // Kept through a reopen during the exit animation, when focus is still inside the closing panel.
      previouslyFocusedEl ??= document.activeElement instanceof HTMLElement ? document.activeElement : null;
      isVisible = true;
    } else if (!open && isVisible && !isClosing) isClosing = true;
  });

  $effect(() => {
    if (!isVisible || isClosing) return;
    void focusInitialElement();
  });

  function restoreFocus() {
    if (!previouslyFocusedEl) return;
    try {
      // The opener can be gone or on its way out (a discarded entry): focus() is then a no-op.
      previouslyFocusedEl.focus();
      reclaimFocus(focusFallback(modalEl));
    } finally {
      previouslyFocusedEl = null;
    }
  }

  $effect(() => {
    if (!isVisible) restoreFocus();
  });

  // Unmounted while open, e.g. its host went away.
  onDestroy(restoreFocus);

  function handleBackdropClick(e: MouseEvent) {
    if (pressedScrim && e.target === e.currentTarget && closeOnBackdrop) onclose?.();
    pressedScrim = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    // A control inside (an open dropdown) already used this key.
    if (e.defaultPrevented || !isTopmost()) return;
    if (closeOnEscape && e.key === "Escape" && isVisible && !isClosing && onclose) {
      // Lets later window listeners see the key was used.
      e.preventDefault();
      onclose();
    }

    if (e.key !== "Tab" || !isVisible || isClosing || !modalEl) return;

    const focusables = getFocusableElements(modalEl);
    if (focusables.length === 0) {
      e.preventDefault();
      modalEl.focus();
      return;
    }

    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;

    if (!active || !modalEl.contains(active)) {
      e.preventDefault();
      first.focus();
      return;
    }

    if (e.shiftKey) {
      if (active === first) {
        e.preventDefault();
        last.focus();
      }
      return;
    }

    if (active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  $effect(() => {
    if (!isVisible || isClosing || !modalEl) return;

    function onFocusIn(e: FocusEvent) {
      const target = e.target;
      if (!(target instanceof HTMLElement) || !isTopmost()) return;
      if (modalEl && !modalEl.contains(target)) {
        void focusInitialElement();
      }
    }

    document.addEventListener("focusin", onFocusIn);
    return () => document.removeEventListener("focusin", onFocusIn);
  });

  function handleAnimationEnd(e: AnimationEvent) {
    // Animations inside the dialog bubble up here too.
    if (e.target === e.currentTarget && isClosing) {
      isVisible = false;
      isClosing = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isVisible}
  {@const styles = modal({ size, closing: isClosing })}
  <div
    class={styles.scrim()}
    role="presentation"
    onpointerdown={(e) => (pressedScrim = e.target === e.currentTarget)}
    onclick={handleBackdropClick}
  >
    <div
      class={styles.panel()}
      bind:this={modalEl}
      {role}
      tabindex="-1"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={describedBy}
      onanimationend={handleAnimationEnd}
    >
      {@render children()}
    </div>
  </div>
{/if}
