<script lang="ts" module>
  // Only the topmost modal handles keys and focus: stacked focus traps pull focus back and forth forever.
  const openStack: object[] = [];
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import { tick } from "svelte";

  interface Props {
    open: boolean;
    onclose?: () => void;
    size?: "sm" | "md" | "lg" | "xl" | "full";
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    class?: string;
    role?: "dialog" | "alertdialog";
    /** ID of the element that labels this modal (for aria-labelledby) */
    labelledBy?: string;
    children: Snippet;
  }

  let {
    open,
    onclose,
    size = "md",
    closeOnBackdrop = true,
    closeOnEscape = true,
    class: className = "",
    role = "dialog",
    labelledBy,
    children,
  }: Props = $props();

  // Internal state to manage exit animation
  let isVisible = $state(false);
  let isClosing = $state(false);

  let modalEl = $state<HTMLElement | null>(null);
  let previouslyFocusedEl = $state<HTMLElement | null>(null);

  const stackToken = {};
  const isTopmost = () => openStack.at(-1) === stackToken;

  $effect(() => {
    if (!isVisible || isClosing) return;
    openStack.push(stackToken);
    return () => void openStack.splice(openStack.indexOf(stackToken), 1);
  });

  function getFocusableElements(root: HTMLElement): HTMLElement[] {
    // Keep selector intentionally conservative to avoid trapping non-interactive elements.
    const selector = [
      "a[href]",
      "summary",
      "button:not([disabled])",
      "input:not([disabled])",
      "select:not([disabled])",
      "textarea:not([disabled])",
      "[tabindex]:not([tabindex='-1'])",
    ].join(",");

    return Array.from(root.querySelectorAll<HTMLElement>(selector)).filter((el) => {
      // Exclude elements that are not actually focusable/visible.
      if (el.hasAttribute("disabled")) return false;
      if (el.getAttribute("aria-disabled") === "true") return false;
      if (el.closest("[inert]")) return false;
      return el.offsetParent !== null || el === document.activeElement;
    });
  }

  async function focusInitialElement() {
    if (!modalEl) return;
    await tick();

    const focusables = getFocusableElements(modalEl);
    const first = focusables[0];

    if (first) {
      first.focus();
      return;
    }

    // If there are no focusable elements, focus the modal container.
    modalEl.tabIndex = -1;
    modalEl.focus();
  }

  // Track open prop changes to trigger animations
  $effect(() => {
    if (open && !isVisible && !isClosing) {
      // Opening: show immediately
      isVisible = true;
    } else if (!open && isVisible && !isClosing) {
      // Closing: trigger exit animation
      isClosing = true;
    }
  });

  // Focus management (capture on open, restore on fully closed)
  $effect(() => {
    if (!open) return;
    previouslyFocusedEl = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  });

  $effect(() => {
    if (!isVisible || isClosing) return;
    void focusInitialElement();
  });

  $effect(() => {
    if (isVisible) return;
    if (!previouslyFocusedEl) return;
    try {
      // The opener can be gone (a discarded entry): fall back to the dialog still open beneath.
      const fallback = [...document.querySelectorAll<HTMLElement>('[aria-modal="true"]')].at(-1);
      (previouslyFocusedEl.isConnected ? previouslyFocusedEl : fallback)?.focus();
    } finally {
      previouslyFocusedEl = null;
    }
  });

  const sizeClasses: Record<string, string> = {
    sm: "w-full max-w-dialog-sm",
    md: "w-full max-w-dialog-md",
    lg: "w-full max-w-dialog-lg",
    xl: "w-full max-w-dialog-xl",
    full: "h-full w-full max-w-dialog-full",
  };

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget && onclose) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // A control inside (an open dropdown) already used this key.
    if (e.defaultPrevented || !isTopmost()) return;
    if (closeOnEscape && e.key === "Escape" && isVisible && !isClosing && onclose) {
      e.stopPropagation();
      onclose();
    }

    if (e.key !== "Tab" || !isVisible || isClosing || !modalEl) return;

    const focusables = getFocusableElements(modalEl);
    if (focusables.length === 0) {
      e.preventDefault();
      modalEl.tabIndex = -1;
      modalEl.focus();
      return;
    }

    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;

    // If focus escaped somehow, bring it back.
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
  <div
    class="fixed inset-0 z-modal flex items-center justify-center bg-black/40
      p-4 backdrop-blur-xs {isClosing ? 'animate-fade-out' : 'animate-fade-in'}"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="flex max-h-full flex-col overflow-hidden rounded-lg border border-border bg-elevated shadow-dialog {sizeClasses[
        size
      ]} {isClosing ? 'animate-modal-out' : 'animate-modal-in'} {className}"
      bind:this={modalEl}
      {role}
      tabindex="-1"
      aria-modal="true"
      aria-labelledby={labelledBy}
      onanimationend={handleAnimationEnd}
    >
      {@render children()}
    </div>
  </div>
{/if}
