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

  let region = $state<HTMLElement>();
  let hovered = false;
  let focused = $state(false);
  let returnFocus: HTMLElement | null = null;

  // A removed element fires no focusout.
  $effect(() => {
    void toastStore.list;
    if (focused && !region?.contains(document.activeElement)) release();
  });

  const toastEl = (id: string) => region?.querySelector<HTMLElement>(`[data-toast-id="${id}"]`);

  function setHovered(next: boolean) {
    hovered = next;
    toastStore.setPaused(hovered || focused);
  }

  function release() {
    focused = false;
    toastStore.hold(null);
    toastStore.setPaused(hovered);
  }

  function onfocusin(e: FocusEvent) {
    if (!focused && e.relatedTarget instanceof HTMLElement && !region?.contains(e.relatedTarget)) {
      returnFocus = e.relatedTarget;
    }
    focused = true;
    toastStore.setPaused(true);
    toastStore.hold((e.target as Element).closest<HTMLElement>("[data-toast-id]")?.dataset.toastId ?? null);
  }

  function onfocusout(e: FocusEvent) {
    if (!(e.relatedTarget instanceof Node && region?.contains(e.relatedTarget))) release();
  }

  /** Focus goes to a neighbouring toast, else back to where it came from, before the toast animates out. */
  function dismiss(id: string) {
    if (toastEl(id)?.contains(document.activeElement)) {
      const list = toastStore.list;
      const i = list.findIndex((t) => t.id === id);
      const next = list[i + 1] ?? list[i - 1];
      const target = next ? toastEl(next.id)?.querySelector<HTMLElement>("[data-toast-dismiss]") : returnFocus;
      if (target?.isConnected) target.focus();
      else (document.activeElement as HTMLElement | null)?.blur();
    }
    toastStore.dismiss(id);
  }
</script>

<!-- Announced through the live regions below, which hold only the newest toast's text. -->
<div
  bind:this={region}
  class="fixed top-toast-offset right-4 z-toast flex flex-col gap-2"
  role="region"
  aria-label="Notifications"
  onpointerenter={() => setHovered(true)}
  onpointerleave={() => setHovered(false)}
  {onfocusin}
  {onfocusout}
>
  {#each toastStore.list as toast (toast.id)}
    <div
      data-toast-id={toast.id}
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
              dismiss(toast.id);
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
        data-toast-dismiss
        onclick={() => dismiss(toast.id)}
      />
    </div>
  {/each}
</div>

<!-- Always mounted: a live region must exist before its content arrives. Errors interrupt; the rest wait. -->
{#each [true, false] as assertive (assertive)}
  {@const current = toastStore.announcement?.assertive === assertive ? toastStore.announcement : null}
  <div class="sr-only" role={assertive ? "alert" : "status"} aria-live={assertive ? "assertive" : "polite"}>
    {#if current}
      {#key current.id}<p>{current.text}</p>{/key}
    {/if}
  </div>
{/each}
