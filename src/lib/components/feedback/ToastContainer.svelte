<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { card, IconButton, LinkButton } from "$lib/components/ui";
  import { type IconName, TONE_FILL, TONE_TEXT } from "$lib/design";
  import { toastStore, type ToastType } from "$lib/stores/toast.svelte";
  import { reflow, shift } from "$lib/utils/motion";

  const ICON: Record<ToastType, IconName> = {
    success: "mdi:check-circle",
    error: "mdi:alert-circle",
    warning: "mdi:alert",
    info: "mdi:information",
  };
</script>

<!-- Always mounted: a live region must exist before its content arrives, and the last toast still animates out.
     The only live region: a role on each toast would nest a second one. -->
<div
  class="fixed top-toast-offset right-4 z-toast flex flex-col gap-2"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
>
  {#each toastStore.list as toast (toast.id)}
    <div
      class={card({
        elevation: "flyout",
        class: "relative flex w-toast items-start gap-3 overflow-hidden py-3 pr-2 pl-4",
      })}
      transition:shift={{ from: "right", by: "lg" }}
      animate:reflow
    >
      <span class="absolute inset-y-0 left-0 w-1 {TONE_FILL[toast.type]}" aria-hidden="true"></span>
      <Icon icon={ICON[toast.type]} size="lg" class="mt-px shrink-0 {TONE_TEXT[toast.type]}" />
      <div class="min-w-0 flex-1">
        {#if toast.subject}
          <div class="text-xs font-medium text-foreground-muted">{toast.subject}</div>
        {/if}
        <div class="text-ui wrap-break-word text-foreground">{toast.message}</div>
        {#if toast.action}
          {@const action = toast.action}
          <LinkButton
            variant="hover"
            tone="accent"
            class="mt-1.5 text-sm"
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
