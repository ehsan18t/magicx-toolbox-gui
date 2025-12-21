<script lang="ts">
  import type { Snippet } from "svelte";

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

  const {
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
  }

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
      {role}
      aria-modal="true"
      aria-labelledby={labelledBy}
      onanimationend={handleAnimationEnd}
    >
      {@render children()}
    </div>
  </div>
{/if}
