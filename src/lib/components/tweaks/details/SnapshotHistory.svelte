<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Callout, PanelHeading, Spinner } from "$lib/components/ui";
  import type { SnapshotHistory } from "$lib/stores/snapshotHistory.svelte";
  import type { EntrySummary } from "$lib/types";
  import { formatDate } from "$lib/utils/time";
  import { expand } from "$lib/utils/motion";

  interface Props {
    history: SnapshotHistory;
    class?: string;
  }

  let { history, class: className }: Props = $props();

  const headingId = $props.id();

  function entryValidity(entry: EntrySummary): string {
    return entry.validity === "Valid" ? "Valid" : `Invalid · ${entry.validity.Invalid}`;
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
          {@const busy = history.busySeq === entry.seq}
          {@const time = formatDate(entry.timestamp, { time: "seconds" })}
          <div
            class="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2"
            transition:expand
          >
            <div class="min-w-0 text-xs wrap-break-word">
              <span class="font-semibold">#{entry.seq}</span>
              <span class="text-foreground-muted"> · {entryValidity(entry)}</span>
              {#if time}<span class="text-foreground-muted"> · {time}</span>{/if}
            </div>
            <button
              type="button"
              class="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded-md px-2 py-1 text-xs font-medium text-foreground-muted hover:bg-error/10 hover:text-error disabled:cursor-not-allowed disabled:opacity-50"
              onclick={() => history.discardWithConfirm(entry.seq)}
              disabled={busy}
              aria-label="Discard snapshot entry {entry.seq}"
            >
              <Icon icon={busy ? "mdi:loading" : "mdi:delete-outline"} size="xs" class={busy ? "animate-spin" : ""} />
              Discard
            </button>
          </div>
        {/each}
      </div>
    {:else if !history.error}
      <p class="m-0 text-ui text-foreground-muted italic">No snapshot entries.</p>
    {/if}
  {/if}
</section>
