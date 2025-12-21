<script lang="ts">
  import type { Snippet } from "svelte";
  import { tick } from "svelte";

  interface Props {
    open: boolean;
    onclose?: () => void;
    size?: "sm" | "md" | "lg" | "xl";
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

  function getFocusableElements(root: HTMLElement): HTMLElement[] {
    // Keep selector intentionally conservative to avoid trapping non-interactive elements.
    const selector = [
      "a[href]",
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
      previouslyFocusedEl.focus();
    } finally {
      previouslyFocusedEl = null;
    }
  });

  const sizeClasses: Record<string, string> = {
    sm: "w-[min(90vw,360px)]",
    md: "w-[min(90vw,480px)]",
    lg: "w-[min(92vw,640px)]",
    xl: "w-[min(92vw,900px)]",
  };

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget && onclose) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
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
      if (!(target instanceof HTMLElement)) return;
      if (modalEl && !modalEl.contains(target)) {
        void focusInitialElement();
      }
    }

    document.addEventListener("focusin", onFocusIn);
    return () => document.removeEventListener("focusin", onFocusIn);
  });

  function handleAnimationEnd(e: AnimationEvent) {
    // Only handle our exit animation
    if (e.animationName === "modal-out" && isClosing) {
      isVisible = false;
      isClosing = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isVisible}
  <div
    class="modal-backdrop fixed inset-0 z-1000 flex items-center justify-center backdrop-blur-sm
      {isClosing ? 'animate-fade-out bg-black/0' : 'animate-fade-in bg-black/60'}"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="modal-content overflow-hidden rounded-xl border border-border bg-card shadow-xl {sizeClasses[
        size
      ]} {isClosing ? 'animate-modal-out' : 'animate-modal-in'} {className}"
      bind:this={modalEl}
      {role}
      aria-modal="true"
      aria-labelledby={labelledBy}
      onanimationend={handleAnimationEnd}
    >
      {@render children()}
    </div>
  </div>
{/if}
