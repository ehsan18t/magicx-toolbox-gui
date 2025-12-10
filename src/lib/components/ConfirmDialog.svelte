<script lang="ts">
  import Icon from "@iconify/svelte";

  interface Props {
    open: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: "default" | "warning" | "danger";
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
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="dialog-backdrop" role="presentation" onclick={handleBackdropClick}>
    <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="dialog-title">
      <div class="dialog-header">
        {#if variant === "warning"}
          <Icon icon="mdi:alert" width="24" class="icon warning" />
        {:else if variant === "danger"}
          <Icon icon="mdi:alert-octagon" width="24" class="icon danger" />
        {:else}
          <Icon icon="mdi:help-circle" width="24" class="icon default" />
        {/if}
        <h2 id="dialog-title" class="dialog-title">{title}</h2>
      </div>

      <div class="dialog-body">
        <p>{message}</p>
      </div>

      <div class="dialog-actions">
        <button class="btn btn-secondary" onclick={oncancel}>
          {cancelText}
        </button>
        <button class="btn btn-{variant}" onclick={onconfirm}>
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(2px);
  }

  .dialog {
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 12px;
    width: min(90vw, 400px);
    box-shadow:
      0 20px 25px -5px rgb(0 0 0 / 0.1),
      0 8px 10px -6px rgb(0 0 0 / 0.1);
    animation: dialog-enter 0.2s ease-out;
  }

  @keyframes dialog-enter {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .dialog-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    border-bottom: 1px solid hsl(var(--border));
  }

  .dialog-header :global(.icon) {
    flex-shrink: 0;
  }

  .dialog-header :global(.icon.warning) {
    color: hsl(45 93% 47%);
  }

  .dialog-header :global(.icon.danger) {
    color: hsl(0 84% 60%);
  }

  .dialog-header :global(.icon.default) {
    color: hsl(var(--primary));
  }

  .dialog-title {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    color: hsl(var(--foreground));
  }

  .dialog-body {
    padding: 16px 20px;
  }

  .dialog-body p {
    margin: 0;
    font-size: 14px;
    color: hsl(var(--muted-foreground));
    line-height: 1.5;
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid hsl(var(--border));
    background: hsl(var(--muted) / 0.3);
    border-radius: 0 0 12px 12px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: none;
  }

  .btn-secondary {
    background: hsl(var(--muted));
    color: hsl(var(--foreground));
  }

  .btn-secondary:hover {
    background: hsl(var(--muted) / 0.8);
  }

  .btn-default {
    background: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
  }

  .btn-default:hover {
    background: hsl(var(--primary) / 0.9);
  }

  .btn-warning {
    background: hsl(45 93% 47%);
    color: hsl(0 0% 0%);
  }

  .btn-warning:hover {
    background: hsl(45 93% 40%);
  }

  .btn-danger {
    background: hsl(0 84% 60%);
    color: white;
  }

  .btn-danger:hover {
    background: hsl(0 84% 50%);
  }
</style>
