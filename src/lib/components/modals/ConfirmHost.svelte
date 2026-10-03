<script lang="ts">
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  const shown = $derived(confirmStore.shown);
</script>

<ConfirmDialog
  {...shown}
  open={confirmStore.current !== null}
  onconfirm={() => confirmStore.settle(true)}
  oncancel={() => confirmStore.settle(false)}
/>
