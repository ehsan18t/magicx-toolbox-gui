<script lang="ts">
  import { toastStore, type ToastType } from "$lib/stores/toast.svelte";
  import Icon from "./Icon.svelte";

  const toasts = $derived(toastStore.list);

  const typeConfig: Record<ToastType, { icon: string; color: string; bgColor: string }> = {
    success: {
      icon: "mdi:check-circle",
      color: "text-success",
      bgColor: "bg-success/10 border-success/20",
    },
    error: {
      icon: "mdi:alert-circle",
      color: "text-error",
      bgColor: "bg-error/10 border-error/20",
    },
    warning: {
      icon: "mdi:alert",
      color: "text-warning",
      bgColor: "bg-warning/10 border-warning/20",
    },
    info: {
      icon: "mdi:information",
      color: "text-accent",
      bgColor: "bg-accent/10 border-accent/20",
    },
  };

  function dismiss(id: string) {
    toastStore.dismiss(id);
  }
</script>

{#if toasts.length > 0}
  <div
    class="fixed right-4 bottom-4 z-1000 flex flex-col gap-2"
    role="region"
    aria-label="Notifications"
    aria-live="polite"
  >
    {#each toasts as toast (toast.id)}
      {@const config = typeConfig[toast.type]}
      <div
        class="flex w-80 items-start gap-3 rounded-lg border p-3 shadow-lg backdrop-blur-sm transition-all duration-300 {config.bgColor} animate-in"
        role="alert"
      >
        <Icon icon={config.icon} width="20" class="{config.color} mt-0.5 shrink-0" />
        <div class="min-w-0 flex-1">
          {#if toast.tweakName}
            <div class="text-xs font-medium text-foreground-muted">{toast.tweakName}</div>
          {/if}
          <div class="text-sm text-foreground">{toast.message}</div>
        </div>
        <button
          class="shrink-0 cursor-pointer rounded p-1 text-foreground-muted transition-colors hover:bg-surface hover:text-foreground"
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
</style>
