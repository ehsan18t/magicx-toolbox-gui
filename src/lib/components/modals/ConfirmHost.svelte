<script lang="ts">
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  const shown = $derived(confirmStore.shown);
</script>

<ConfirmDialog
  open={confirmStore.current !== null}
  title={shown.title}
  message={shown.message}
  confirmText={shown.confirmText}
  cancelText={shown.cancelText}
  variant={shown.variant}
  onconfirm={() => confirmStore.settle(true)}
  oncancel={() => confirmStore.settle(false)}
/>
