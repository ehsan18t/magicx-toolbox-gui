<script lang="ts" module>
  // Only the topmost modal handles keys and focus: stacked focus traps pull focus back and forth forever.
  const openStack: object[] = [];
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import { tick } from "svelte";

  type Size = "sm" | "md" | "lg" | "xl" | "full";

  interface Props {
    open: boolean;
    onclose?: () => void;
    size?: Size;
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    role?: "dialog" | "alertdialog";
    labelledBy?: string;
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
    labelledBy,
    describedBy,
    children,
  }: Props = $props();

  const SIZE_CLASS: Record<Size, string> = {
    sm: "max-w-dialog-sm",
    md: "max-w-dialog-md",
    lg: "max-w-dialog-lg",
    xl: "max-w-dialog-xl",
    full: "h-full max-w-dialog-full",
  };

  // Mounted while open or playing the exit animation.
  let isVisible = $state(false);
  let isClosing = $state(false);

  let modalEl = $state<HTMLElement | null>(null);
  let previouslyFocusedEl: HTMLElement | null = null;

  const stackToken = {};
  const isTopmost = () => openStack.at(-1) === stackToken;

  $effect(() => {
    if (!isVisible || isClosing) return;
    openStack.push(stackToken);
    return () => void openStack.splice(openStack.indexOf(stackToken), 1);
  });

  function getFocusableElements(root: HTMLElement): HTMLElement[] {
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
      if (el.hasAttribute("disabled")) return false;
      if (el.getAttribute("aria-disabled") === "true") return false;
      if (el.closest("[inert]")) return false;
      return el.offsetParent !== null || el === document.activeElement;
    });
  }

  async function focusInitialElement() {
    if (!modalEl) return;
    await tick();

    (getFocusableElements(modalEl)[0] ?? modalEl).focus();
  }

  $effect(() => {
    if (open && !isVisible && !isClosing) isVisible = true;
    else if (!open && isVisible && !isClosing) isClosing = true;
  });

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

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget && onclose) {
      onclose();
    }
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
  <div
    class="fixed inset-0 z-modal flex items-center justify-center bg-black/40
      p-4 backdrop-blur-xs {isClosing ? 'animate-fade-out' : 'animate-fade-in'}"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="flex max-h-full w-full flex-col overflow-hidden rounded-lg border border-border bg-elevated shadow-dialog {SIZE_CLASS[
        size
      ]} {isClosing ? 'animate-modal-out' : 'animate-modal-in'}"
      bind:this={modalEl}
      {role}
      tabindex="-1"
      aria-modal="true"
      aria-labelledby={labelledBy}
      aria-describedby={describedBy}
      onanimationend={handleAnimationEnd}
    >
      {@render children()}
    </div>
  </div>
{/if}
