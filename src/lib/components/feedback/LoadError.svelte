<script lang="ts">
  import { Icon } from "$lib/components/shared";

  let { message }: { message: string } = $props();
</script>

<div class="w-full max-w-sm rounded-lg border border-border bg-card p-6 text-center">
  <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-error/15 text-error">
    <Icon icon="mdi:alert-circle" width="28" />
  </div>
  <h2 class="mt-4 mb-1 text-base font-semibold">Failed to load</h2>
  <p class="m-0 text-sm wrap-break-word text-foreground-muted">{message}</p>
  <button
    type="button"
    class="mt-5 inline-flex w-full cursor-pointer items-center justify-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-semibold text-accent-foreground hover:bg-accent-hover"
    onclick={() => window.location.reload()}
  >
    <Icon icon="mdi:refresh" width="18" />
    Retry
  </button>
</div>
