<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { type IconName, type Tone, TONE_TEXT } from "$lib/design";
  import { Button, Modal, ModalBody, ModalFooter, ModalHeader } from "$lib/components/ui";
  import type { ButtonVariants } from "$lib/components/ui/variants";
  import type { ConfirmVariant as Variant } from "$lib/stores/confirm.svelte";

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

  let {
    open,
    title,
    message,
    confirmText = "Confirm",
    cancelText = "Cancel",
    variant = "default",
    onconfirm,
    oncancel,
  }: Props = $props();

  const VARIANT: Record<Variant, { icon: IconName; tone: Tone; button: ButtonVariants["variant"] }> = {
    default: { icon: "mdi:help-circle", tone: "accent", button: "primary" },
    warning: { icon: "mdi:alert", tone: "warning", button: "warning" },
    danger: { icon: "mdi:alert-octagon", tone: "error", button: "danger" },
  };

  const config = $derived(VARIANT[variant]);
  const messageId = $props.id();
</script>

<Modal {open} onclose={oncancel} size="sm" role="alertdialog" describedBy={messageId}>
  <ModalHeader {title}>
    {#snippet leading()}<Icon icon={config.icon} size="3xl" class="shrink-0 {TONE_TEXT[config.tone]}" />{/snippet}
  </ModalHeader>

  <ModalBody>
    <p id={messageId} class="m-0 text-sm leading-relaxed text-foreground-muted">{message}</p>
  </ModalBody>

  <ModalFooter>
    <Button variant="secondary" onclick={oncancel}>{cancelText}</Button>
    <Button variant={config.button} onclick={onconfirm}>{confirmText}</Button>
  </ModalFooter>
</Modal>
