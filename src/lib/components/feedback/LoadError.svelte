<script lang="ts">
  import { Button, Card, IconTile } from "$lib/components/ui";
  import { HEADING } from "$lib/design";

  let {
    message,
    title = "Failed to load",
    onretry = () => window.location.reload(),
  }: { message: string; title?: string; onretry?: () => void } = $props();
</script>

<!-- Fills its parent, a block or a flex column, and centres the card in it. -->
<div class="flex h-full min-h-0 flex-1 items-center justify-center p-6">
  <Card class="w-full max-w-sm p-6 text-center" role="alert">
    <IconTile icon="mdi:alert-circle" size="xl" shape="circle" tone="error" class="mx-auto" />
    <h2 class={["mt-4 mb-1", HEADING.section]}>{title}</h2>
    <p class="m-0 text-sm wrap-break-word text-foreground-muted">{message}</p>
    <Button variant="primary" icon="mdi:refresh" class="mt-5 w-full" onclick={onretry}>Retry</Button>
  </Card>
</div>
