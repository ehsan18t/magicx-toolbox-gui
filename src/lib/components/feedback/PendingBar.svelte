<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, Card, IconButton } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweakDetailsModalStore } from "$lib/stores/detailsModal.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { plural } from "$lib/utils/format";
  import { expand, pop, shift } from "$lib/utils/motion";
  import { isHighRisk, stateSummary } from "$lib/utils/tweakPresentation";
  import PendingReviewModal, { type PendingItem } from "./PendingReviewModal.svelte";

  let expanded = $state(false);
  let applying = $state(false);
  let reviewing = $state(false);

  const count = $derived(pendingChangesStore.count);
  const items = $derived(
    Array.from(pendingChangesStore.all.values()).map((change): PendingItem => {
      const tweak = tweaksStore.tweak(change.tweakId);
      const from = tweak ? stateSummary(tweak.status).label : "";
      return {
        change,
        tweak,
        name: tweak?.definition.name ?? change.tweakId,
        fromArrow: from ? `${from} → ` : "→ ",
        highRisk: tweak ? isHighRisk(tweak.definition.riskLevel) : false,
      };
    }),
  );
  const needsReboot = $derived(items.some((i) => i.tweak?.definition.requiresReboot));
  // applyPending marks the store busy synchronously, so `applying` only picks the button that spins.
  const busy = $derived(tweakActionsStore.isBusy);

  const reviewOpen = $derived(reviewing && count > 0);

  // High-risk changes are confirmed here, once, with the whole batch in view, not on each click.
  function requestApply() {
    if (items.some((i) => i.highRisk)) reviewing = true;
    else void apply();
  }

  async function apply() {
    reviewing = false;
    applying = true;
    try {
      await tweakActionsStore.applyPending();
    } finally {
      applying = false;
    }
  }
</script>

{#if count > 0}
  <div class="pointer-events-none absolute inset-x-0 bottom-0 z-dock flex justify-center px-4 pb-4" transition:shift>
    <Card
      elevation="flyout"
      class="pointer-events-auto w-full max-w-xl overflow-hidden"
      role="region"
      aria-label="Pending changes"
    >
      {#if expanded}
        <ul class="m-0 max-h-56 list-none overflow-y-auto border-b border-border p-1" transition:expand>
          {#each items as { change, name, fromArrow } (change.tweakId)}
            <li class="flex items-center gap-2 rounded px-2 py-1.5 hover:bg-muted" transition:expand>
              <button
                type="button"
                class="min-w-0 flex-1 cursor-pointer truncate text-left text-ui"
                onclick={() => tweakDetailsModalStore.open(change.tweakId)}
              >
                <span class="text-foreground">{name}</span>
                <span class="text-foreground-muted"> {fromArrow}{change.optionLabel}</span>
              </button>
              <IconButton
                icon="mdi:close"
                size="xs"
                label="Unstage {name}"
                onclick={() => pendingChangesStore.remove(change.tweakId)}
              />
            </li>
          {/each}
        </ul>
      {/if}

      <div class="flex flex-wrap items-center gap-x-3 gap-y-2 px-3 py-2.5">
        <button
          type="button"
          class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 text-left"
          aria-expanded={expanded}
          onclick={() => (expanded = !expanded)}
        >
          <span
            class="flex h-6 min-w-6 shrink-0 items-center justify-center rounded-full bg-warning px-1.5 text-xs font-bold text-warning-foreground tabular-nums"
          >
            {#key count}<span class="inline-block" in:pop>{count}</span>{/key}
          </span>
          <span class="min-w-0">
            <span class="block truncate text-ui font-semibold text-foreground">
              {plural(count, "change")} ready to apply
            </span>
            {#if needsReboot}
              <span class="block truncate text-xs text-foreground-muted">Some need a restart to take effect</span>
            {/if}
          </span>
          <Icon
            icon="mdi:chevron-up"
            size="md"
            class="shrink-0 text-foreground-muted transition-transform duration-normal {expanded ? 'rotate-180' : ''}"
          />
        </button>

        <div class="ml-auto flex shrink-0 items-center gap-2">
          <Button disabled={busy} onclick={() => pendingChangesStore.clear()}>Discard</Button>
          <Button variant="primary" loading={applying} disabled={busy} onclick={requestApply}>Apply</Button>
        </div>
      </div>
    </Card>
  </div>
{/if}

<PendingReviewModal open={reviewOpen} {items} {busy} onclose={() => (reviewing = false)} onapply={apply} />
