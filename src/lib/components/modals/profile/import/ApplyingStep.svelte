<script lang="ts">
  import { BUSY_HINT } from "$lib/components/feedback";
  import { IconTile, ModalBody, ProgressBar } from "$lib/components/ui";
  import { plural } from "$lib/utils/format";

  let { progress }: { progress: { current: number; total: number } | null } = $props();
</script>

<ModalBody class="flex animate-fade-in flex-col items-center gap-6 py-8 text-center">
  <IconTile icon="mdi:cog" size="3xl" shape="circle" spin />

  <div class="w-full max-w-sm">
    <p class="m-0 mb-4 font-medium">Applying profile changes…</p>
    <ProgressBar value={progress?.current ?? 0} max={progress?.total} label="Applying profile" showValue={!!progress} />
    {#if progress}
      <p class="m-0 mt-2 text-sm text-foreground-muted">{progress.current} of {plural(progress.total, "tweak")}</p>
    {/if}
  </div>

  <p class="m-0 text-sm text-foreground-muted">{BUSY_HINT}</p>
</ModalBody>
