<script lang="ts">
  import { TONE_WASH } from "$lib/design";
  import { Button, Callout, PanelHeading, Spinner } from "$lib/components/ui";
  import type { SnapshotHistory } from "$lib/stores/snapshotHistory.svelte";
  import type { EntrySummary } from "$lib/types";
  import { expand } from "$lib/utils/motion";
  import { formatDate } from "$lib/utils/time";
  import { SEP } from "$lib/utils/tweakPresentation";

  interface Props {
    history: SnapshotHistory;
    class?: string;
  }

  let { history, class: className }: Props = $props();

  const headingId = $props.id();

  function entrySummary(entry: EntrySummary): string {
    const validity = entry.validity === "Valid" ? "Valid" : `Invalid${SEP}${entry.validity.Invalid}`;
    return [validity, formatDate(entry.timestamp, { time: "seconds" })].filter(Boolean).join(SEP);
  }
</script>

<section aria-labelledby={headingId} class={className}>
  <PanelHeading id={headingId} icon="mdi:history" class="mb-2.5">Snapshot history</PanelHeading>
  {#if history.loading}
    <div class="flex items-center gap-2 text-ui text-foreground-muted">
      <Spinner size="sm" class="text-current" />
      Loading…
    </div>
  {:else}
    {#if history.error}
      <Callout tone="error" density="compact" icon="mdi:alert-circle" class="mb-1.5">
        <span>The snapshot entries could not be listed: {history.error}</span>
      </Callout>
    {/if}
    {#if history.entries.length > 0}
      <div class="animate-fade-in space-y-1.5">
        {#each history.entries as entry (entry.seq)}
          <div
            class="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2"
            transition:expand
          >
            <div class="min-w-0 text-xs wrap-break-word">
              <span class="font-semibold">#{entry.seq}</span>
              <span class="text-foreground-muted">{SEP}{entrySummary(entry)}</span>
            </div>
            <Button
              variant="ghost"
              size="sm"
              icon="mdi:delete-outline"
              class={TONE_WASH.error}
              loading={history.isBusy(entry.seq)}
              aria-label="Discard snapshot entry {entry.seq}"
              onclick={() => history.discardWithConfirm(entry.seq)}
            >
              Discard
            </Button>
          </div>
        {/each}
      </div>
    {:else if !history.error}
      <p class="m-0 text-ui text-foreground-muted italic">No snapshot entries.</p>
    {/if}
  {/if}
</section>
