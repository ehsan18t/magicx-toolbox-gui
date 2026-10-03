<script lang="ts">
  let { id, reason }: { id: string; reason: string | null | undefined } = $props();
</script>

<!-- `hidden`, not sr-only: aria-describedby still reads it, and it stays out of the content name. -->
{#if reason}<span {id} hidden>{reason}</span>{/if}
