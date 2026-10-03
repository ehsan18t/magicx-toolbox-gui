<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { IconButton, LinkButton } from "$lib/components/ui";
  import { card } from "$lib/components/ui/variants";
  import { type IconName, TONE_TEXT } from "$lib/design";
  import { toastStore, type ToastType } from "$lib/stores/toast.svelte";
  import { reflow, shift } from "$lib/utils/motion";

  const typeConfig: Record<ToastType, { icon: IconName; stripe: string }> = {
    success: { icon: "mdi:check-circle", stripe: "bg-success" },
    error: { icon: "mdi:alert-circle", stripe: "bg-error" },
    warning: { icon: "mdi:alert", stripe: "bg-warning" },
    info: { icon: "mdi:information", stripe: "bg-info" },
  };
</script>

<!-- Always mounted: a live region must exist before its content arrives, and the last toast still animates out. -->
<div
  class="fixed top-toast-offset right-4 z-toast flex flex-col gap-2"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
>
  {#each toastStore.list as toast (toast.id)}
    {@const config = typeConfig[toast.type]}
    <div
      class={card({
        elevation: "flyout",
        class: "relative flex w-toast items-start gap-3 overflow-hidden py-3 pr-2 pl-4",
      })}
      role={toast.type === "error" ? "alert" : "status"}
      transition:shift={{ from: "right", by: "lg" }}
      animate:reflow
    >
      <span class="absolute inset-y-0 left-0 w-1 {config.stripe}" aria-hidden="true"></span>
      <Icon icon={config.icon} size="lg" class="mt-px shrink-0 {TONE_TEXT[toast.type]}" />
      <div class="min-w-0 flex-1">
        {#if toast.subject}
          <div class="text-xs font-medium text-foreground-muted">{toast.subject}</div>
        {/if}
        <div class="text-ui wrap-break-word text-foreground">{toast.message}</div>
        {#if toast.action}
          {@const action = toast.action}
          <LinkButton
            variant="hover"
            class="mt-1.5 text-sm text-accent"
            onclick={() => {
              action.run();
              toastStore.dismiss(toast.id);
            }}
          >
            {action.label}
          </LinkButton>
        {/if}
      </div>
      <IconButton
        icon="mdi:close"
        size="xs"
        label="Dismiss notification"
        onclick={() => toastStore.dismiss(toast.id)}
      />
    </div>
  {/each}
</div>
