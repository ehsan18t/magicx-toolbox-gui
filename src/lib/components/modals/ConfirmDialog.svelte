<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, Modal, ModalBody, ModalFooter, ModalHeader } from "$lib/components/ui";

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

  const variantConfig: Record<
    Variant,
    { icon: string; iconColor: string; buttonVariant: "primary" | "warning" | "danger" }
  > = {
    default: { icon: "mdi:help-circle", iconColor: "text-accent", buttonVariant: "primary" },
    warning: { icon: "mdi:alert", iconColor: "text-warning", buttonVariant: "warning" },
    danger: { icon: "mdi:alert-octagon", iconColor: "text-error", buttonVariant: "danger" },
  };

  const config = $derived(variantConfig[variant]);
</script>

<Modal {open} onclose={oncancel} size="sm" role="alertdialog" labelledBy="confirm-dialog-title">
  <ModalHeader id="confirm-dialog-title">
    <div class="flex items-center gap-3">
      <Icon icon={config.icon} width="24" class="shrink-0 {config.iconColor}" />
      <h2 class="m-0 text-base font-semibold text-foreground">{title}</h2>
    </div>
  </ModalHeader>

  <ModalBody>
    <p class="m-0 text-sm leading-relaxed text-foreground-muted">{message}</p>
  </ModalBody>

  <ModalFooter>
    <Button variant="secondary" onclick={oncancel}>{cancelText}</Button>
    <Button variant={config.buttonVariant} onclick={onconfirm}>{confirmText}</Button>
  </ModalFooter>
</Modal>
