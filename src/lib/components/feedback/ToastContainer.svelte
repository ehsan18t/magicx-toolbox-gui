<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { toastStore, type ToastType } from "$lib/stores/toast.svelte";
  import { pendingRebootStore } from "$lib/stores/tweaksPending.svelte";
  import { reflow, shift } from "$lib/utils/motion";

  const typeConfig: Record<ToastType, { icon: string; color: string; stripe: string }> = {
    success: { icon: "mdi:check-circle", color: "text-success", stripe: "bg-success" },
    error: { icon: "mdi:alert-circle", color: "text-error", stripe: "bg-error" },
    warning: { icon: "mdi:alert", color: "text-warning", stripe: "bg-warning" },
    info: { icon: "mdi:information", color: "text-info", stripe: "bg-info" },
  };
</script>

<!-- Always mounted: a live region must exist before its content arrives, and the last toast still animates out. -->
<div
  class="fixed right-4 z-toast flex flex-col gap-2 transition-[top] duration-slow {pendingRebootStore.count > 0
    ? 'top-toast-offset-banner'
    : 'top-toast-offset'}"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
>
  {#each toastStore.list as toast (toast.id)}
    {@const config = typeConfig[toast.type]}
    <div
      class="relative flex w-toast items-start gap-3 overflow-hidden rounded-lg border border-border bg-elevated py-3 pr-2 pl-4 shadow-flyout"
      role={toast.type === "error" ? "alert" : "status"}
      transition:shift={{ from: "right", by: "lg" }}
      animate:reflow
    >
      <span class="absolute inset-y-0 left-0 w-1 {config.stripe}" aria-hidden="true"></span>
      <Icon icon={config.icon} width="18" class="mt-px shrink-0 {config.color}" />
      <div class="min-w-0 flex-1">
        {#if toast.subject}
          <div class="text-xs font-medium text-foreground-muted">{toast.subject}</div>
        {/if}
        <div class="text-ui wrap-break-word text-foreground">{toast.message}</div>
        {#if toast.action}
          {@const action = toast.action}
          <button
            type="button"
            class="mt-1.5 cursor-pointer rounded border-0 bg-transparent p-0 text-sm font-medium text-accent underline-offset-2 hover:underline"
            onclick={() => {
              action.run();
              toastStore.dismiss(toast.id);
            }}
          >
            {action.label}
          </button>
        {/if}
      </div>
      <button
        class="shrink-0 cursor-pointer rounded border-0 bg-transparent p-1 text-foreground-muted hover:bg-muted hover:text-foreground"
        onclick={() => toastStore.dismiss(toast.id)}
        aria-label="Dismiss notification"
      >
        <Icon icon="mdi:close" width="16" />
      </button>
    </div>
  {/each}
</div>
