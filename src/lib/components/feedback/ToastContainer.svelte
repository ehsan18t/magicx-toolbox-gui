<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { toastStore, type ToastType } from "$lib/stores/toast.svelte";
  import { pendingRebootStore } from "$lib/stores/tweaks.svelte";

  const toasts = $derived(toastStore.list);

  // Track which toasts are being dismissed for exit animation
  let dismissingIds = $state<Set<string>>(new Set());

  const typeConfig: Record<ToastType, { icon: string; color: string; stripe: string }> = {
    success: { icon: "mdi:check-circle", color: "text-success", stripe: "bg-success" },
    error: { icon: "mdi:alert-circle", color: "text-error", stripe: "bg-error" },
    warning: { icon: "mdi:alert", color: "text-warning", stripe: "bg-warning" },
    info: { icon: "mdi:information", color: "text-info", stripe: "bg-info" },
  };

  function dismiss(id: string) {
    // Start exit animation
    dismissingIds = new Set([...dismissingIds, id]);

    // Actually remove after animation
    setTimeout(() => {
      toastStore.dismiss(id);
      dismissingIds = new Set([...dismissingIds].filter((d) => d !== id));
    }, 200);
  }
</script>

{#if toasts.length > 0}
  <div
    class="fixed right-4 z-toast flex flex-col gap-2 {pendingRebootStore.count > 0 ? 'top-24' : 'top-14'}"
    role="region"
    aria-label="Notifications"
    aria-live="polite"
  >
    {#each toasts as toast (toast.id)}
      {@const config = typeConfig[toast.type]}
      {@const isDismissing = dismissingIds.has(toast.id)}
      <div
        class="relative flex w-[min(22rem,calc(100vw-2rem))] items-start gap-3 overflow-hidden rounded-lg border border-border bg-elevated py-3 pr-2 pl-4 shadow-flyout
          {isDismissing ? 'animate-out' : 'animate-in'}"
        role={toast.type === "error" ? "alert" : "status"}
      >
        <span class="absolute inset-y-0 left-0 w-1 {config.stripe}" aria-hidden="true"></span>
        <Icon icon={config.icon} width="18" class="mt-px shrink-0 {config.color}" />
        <div class="min-w-0 flex-1">
          {#if toast.tweakName}
            <div class="text-xs font-medium text-foreground-muted">{toast.tweakName}</div>
          {/if}
          <div class="text-[13px] wrap-break-word text-foreground">{toast.message}</div>
          {#if toast.action}
            {@const action = toast.action}
            <button
              type="button"
              class="mt-1.5 cursor-pointer rounded border-0 bg-transparent p-0 text-sm font-medium text-accent underline-offset-2 hover:underline"
              onclick={() => {
                action.run();
                dismiss(toast.id);
              }}
            >
              {action.label}
            </button>
          {/if}
        </div>
        <button
          class="shrink-0 cursor-pointer rounded border-0 bg-transparent p-1 text-foreground-muted transition-colors hover:bg-muted hover:text-foreground"
          onclick={() => dismiss(toast.id)}
          aria-label="Dismiss notification"
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .animate-in {
    animation: slide-in 0.3s ease-out;
  }

  .animate-out {
    animation: slide-out 0.2s ease-in forwards;
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(100%);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @keyframes slide-out {
    from {
      opacity: 1;
      transform: translateX(0);
    }
    to {
      opacity: 0;
      transform: translateX(100%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .animate-in,
    .animate-out {
      animation: none;
    }
  }
</style>
