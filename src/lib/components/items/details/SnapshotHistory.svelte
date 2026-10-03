<script lang="ts">
  import { TONE_WASH } from "$lib/design";
  import { Button, Callout, card, PanelSection, Spinner } from "$lib/components/ui";
  import type { SnapshotHistory } from "$lib/stores/snapshotHistory.svelte";
  import type { EntrySummary } from "$lib/types";
  import { reclaimFocus } from "$lib/utils/focus";
  import { SEP } from "$lib/utils/format";
  import { expand } from "$lib/utils/motion";
  import { formatDate } from "$lib/utils/time";

  interface Props {
    history: SnapshotHistory;
    class?: string;
  }

  let { history, class: className }: Props = $props();

  const id = $props.id();
  const discardId = (seq: number) => `${id}-discard-${seq}`;

  function entrySummary(entry: EntrySummary): string {
    const validity = entry.validity === "Valid" ? "Valid" : `Invalid${SEP}${entry.validity.Invalid}`;
    return [validity, formatDate(entry.timestamp, { time: "seconds" })].filter(Boolean).join(SEP);
  }

  // Focus moves to the entry that took the discarded one's place, else the one above, else the dialog.
  // Moved outright: the closing confirmation may still hold focus, and its restore finds the opener gone.
  async function discard(seq: number, index: number) {
    await history.discardWithConfirm(seq);
    if (history.entries.some((entry) => entry.seq === seq)) return;
    const next = history.entries[index] ?? history.entries[index - 1];
    const button = next && document.getElementById(discardId(next.seq));
    if (button) button.focus();
    else reclaimFocus();
  }
</script>

<PanelSection title="Snapshot history" icon="mdi:history" class={className}>
  {#if history.loading}
    <Spinner size="md" tone="current" class="flex text-ui text-foreground-muted">Loading…</Spinner>
  {:else}
    {#if history.error}
      <Callout tone="error" density="compact" icon="mdi:alert-circle" class="mb-1.5">
        <span>The snapshot entries could not be listed: {history.error}</span>
      </Callout>
    {/if}
    {#if history.entries.length > 0}
      <div class="animate-fade-in space-y-1.5">
        {#each history.entries as entry, i (entry.seq)}
          <div
            class={card({ radius: "md", class: "flex items-center justify-between gap-3 px-3 py-2" })}
            transition:expand
          >
            <div class="min-w-0 text-xs wrap-break-word">
              <span class="font-semibold">#{entry.seq}</span>
              <span class="text-foreground-muted">{SEP}{entrySummary(entry)}</span>
            </div>
            <Button
              id={discardId(entry.seq)}
              variant="ghost"
              size="sm"
              icon="mdi:delete-outline"
              class={TONE_WASH.error}
              loading={history.isBusy(entry.seq)}
              aria-label="Discard snapshot entry {entry.seq}"
              onclick={() => discard(entry.seq, i)}
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
</PanelSection>
