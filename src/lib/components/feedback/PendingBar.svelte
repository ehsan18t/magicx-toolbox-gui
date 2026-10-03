<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, IconButton, Modal, ModalBody, ModalFooter, ModalHeader } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { plural } from "$lib/utils/format";
  import { expand, pop, shift } from "$lib/utils/motion";
  import { isHighRisk, RISK_INFO, stateSummary } from "$lib/utils/tweakPresentation";

  let expanded = $state(false);
  let applying = $state(false);
  let reviewing = $state(false);

  const count = $derived(pendingChangesStore.count);
  const items = $derived(
    Array.from(pendingChangesStore.all.values()).map((change) => {
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
  const highRiskCount = $derived(items.filter((i) => i.highRisk).length);
  // applyPending marks the store busy synchronously, so `applying` only picks the button that spins.
  const busy = $derived(tweakActionsStore.isBusy);

  const reviewOpen = $derived(reviewing && count > 0);

  // High-risk changes are confirmed here, once, with the whole batch in view, not on each click.
  function requestApply() {
    if (highRiskCount > 0) reviewing = true;
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
    <div
      class="pointer-events-auto w-full max-w-xl overflow-hidden rounded-lg border border-border bg-elevated shadow-flyout"
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
          <Button
            variant="primary"
            class="px-3.5 font-semibold"
            loading={applying}
            disabled={busy}
            onclick={requestApply}
          >
            Apply
          </Button>
        </div>
      </div>
    </div>
  </div>
{/if}

<Modal open={reviewOpen} onclose={() => (reviewing = false)} size="lg">
  <ModalHeader title="Review {plural(count, 'change')}">
    {#snippet leading()}<Icon icon="mdi:alert" size="2xl" class="shrink-0 text-warning" />{/snippet}
    <p class="m-0 mt-0.5 text-ui text-foreground-muted">
      {plural(highRiskCount, "change")}
      {highRiskCount === 1 ? "is" : "are"} high risk. Check them before applying.
    </p>
  </ModalHeader>
  <ModalBody>
    <ul class="m-0 list-none space-y-1.5 p-0">
      {#each items as { change, tweak, name, fromArrow, highRisk } (change.tweakId)}
        <li class="rounded-md border px-3 py-2 text-ui {highRisk ? 'border-error/35 bg-error/6' : 'border-border'}">
          <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
            <span class="font-medium text-foreground">{name}</span>
            <span class="text-foreground-muted">
              {fromArrow}<span class="font-medium text-foreground">{change.optionLabel}</span>
            </span>
          </div>
          {#if tweak && (highRisk || tweak.definition.requiresReboot)}
            <div class="mt-1 flex flex-wrap gap-x-3 text-xs">
              {#if highRisk}
                <span class="text-error">{RISK_INFO[tweak.definition.riskLevel].name} risk</span>
              {/if}
              {#if tweak.definition.requiresReboot}<span class="text-info">Needs a restart</span>{/if}
            </div>
          {/if}
          {#if highRisk && tweak?.definition.warning}
            <p class="m-0 mt-1 text-xs leading-relaxed text-foreground-muted">{tweak.definition.warning}</p>
          {/if}
        </li>
      {/each}
    </ul>
  </ModalBody>
  <ModalFooter>
    <Button variant="secondary" onclick={() => (reviewing = false)}>Back</Button>
    <Button variant="warning" onclick={apply} disabled={busy}
      >Apply {count === 1 ? "change" : `${count} changes`}</Button
    >
  </ModalFooter>
</Modal>
