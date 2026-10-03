<script lang="ts">
  import { Button, Card, IconTile } from "$lib/components/ui";

  let { message }: { message: string } = $props();
</script>

<!-- Fills its parent, a block or a flex column, and centres the card in it. -->
<div class="flex h-full min-h-0 flex-1 items-center justify-center p-6">
  <Card class="w-full max-w-sm p-6 text-center" role="alert">
    <IconTile icon="mdi:alert-circle" size="xl" shape="circle" tone="error" class="mx-auto" />
    <h2 class="mt-4 mb-1 text-base font-semibold">Failed to load</h2>
    <p class="m-0 text-sm wrap-break-word text-foreground-muted">{message}</p>
    <Button variant="primary" icon="mdi:refresh" class="mt-5 w-full" onclick={() => window.location.reload()}
      >Retry</Button
    >
  </Card>
</div>
