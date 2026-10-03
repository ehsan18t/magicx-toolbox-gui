<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, Card, ICON_SIZE, TONE_SOFT } from "$lib/components/ui";

  let { message }: { message: string } = $props();
</script>

<Card class="w-full max-w-sm p-6 text-center" role="alert">
  <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full {TONE_SOFT.error}">
    <Icon icon="mdi:alert-circle" width={28} />
  </div>
  <h2 class="mt-4 mb-1 text-base font-semibold">Failed to load</h2>
  <p class="m-0 text-sm wrap-break-word text-foreground-muted">{message}</p>
  <Button variant="primary" class="mt-5 w-full" onclick={() => window.location.reload()}>
    <Icon icon="mdi:refresh" width={ICON_SIZE.md} />
    Retry
  </Button>
</Card>
