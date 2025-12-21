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
    if (closeOnEscape && e.key === "Escape" && open && onclose) {
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="modal-backdrop fixed inset-0 z-1000 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="modal-content animate-modal-in overflow-hidden rounded-xl border border-border bg-card shadow-xl {sizeClasses[
        size
      ]} {className}"
      {role}
      aria-modal="true"
      aria-labelledby={labelledBy}
    >
      {@render children()}
    </div>
  </div>
{/if}
