<script lang="ts">
  import Icon from "./Icon.svelte";

  type Variant = "default" | "warning" | "danger";

  interface Props {
    open: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: Variant;
    onconfirm: () => void;
    oncancel: () => void;
  }

  const {
    open,
    title,
    message,
    confirmText = "Confirm",
    cancelText = "Cancel",
    variant = "default",
    onconfirm,
    oncancel,
  }: Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      oncancel();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      oncancel();
    }
  }

  const iconClasses: Record<Variant, { icon: string; color: string }> = {
    default: { icon: "mdi:help-circle", color: "text-accent" },
    warning: { icon: "mdi:alert", color: "text-warning" },
    danger: { icon: "mdi:alert-octagon", color: "text-error" },
  };

  const confirmBtnClasses: Record<Variant, string> = {
    default: "bg-accent text-accent-foreground hover:bg-accent/90",
    warning: "bg-warning text-black hover:bg-warning/90",
    danger: "bg-error text-white hover:bg-error/90",
  };
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 z-1000 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="animate-in zoom-in-95 w-[min(90vw,400px)] rounded-xl border border-border bg-card shadow-xl duration-200"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
    >
      <!-- Header -->
      <div class="flex items-center gap-3 border-b border-border px-5 py-4">
        <Icon
          icon={iconClasses[variant].icon}
          width="24"
          class="shrink-0 {iconClasses[variant].color}"
        />
        <h2 id="dialog-title" class="m-0 text-base font-semibold text-foreground">{title}</h2>
      </div>

      <!-- Body -->
      <div class="px-5 py-4">
        <p class="m-0 text-sm leading-relaxed text-foreground-muted">{message}</p>
      </div>

      <!-- Actions -->
      <div
        class="flex justify-end gap-2 rounded-b-xl border-t border-border bg-[hsl(var(--muted)/0.3)] px-5 py-3"
      >
        <button
          class="cursor-pointer rounded-md border-0 bg-[hsl(var(--muted))] px-4 py-2 text-sm font-medium text-foreground transition-all duration-150 hover:bg-[hsl(var(--muted)/0.8)]"
          onclick={oncancel}
        >
          {cancelText}
        </button>
        <button
          class="cursor-pointer rounded-md border-0 px-4 py-2 text-sm font-medium transition-all duration-150 {confirmBtnClasses[
            variant
          ]}"
          onclick={onconfirm}
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes zoom-in-95 {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .animate-in {
    animation: zoom-in-95 0.2s ease-out;
  }
</style>
