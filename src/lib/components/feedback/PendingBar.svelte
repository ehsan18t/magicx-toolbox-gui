<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, Modal, ModalBody, ModalFooter, ModalHeader } from "$lib/components/ui";
  import { tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { tweakOps } from "$lib/stores/tweakOps.svelte";
  import { applyPendingChanges } from "$lib/stores/tweaksActions.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import { expand, pop, shift } from "$lib/utils/motion";
  import { isHighRisk, RISK_INFO, stateSummary } from "$lib/utils/tweakPresentation";

  let expanded = $state(false);
  let applying = $state(false);
  let reviewing = $state(false);

  const count = $derived(pendingChangesStore.count);
  const items = $derived(
    Array.from(pendingChangesStore.all.values()).map((change) => {
      const tweak = tweaksStore.getById(change.tweakId);
      return {
        change,
        tweak,
        from: tweak ? stateSummary(tweak.status).label : "",
        highRisk: tweak ? isHighRisk(tweak.definition.riskLevel) : false,
      };
    }),
  );
  const needsReboot = $derived(items.some((i) => i.tweak?.definition.requiresReboot));
  const highRiskCount = $derived(items.filter((i) => i.highRisk).length);
  const busy = $derived(applying || tweakOps.isBusy);

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
      await applyPendingChanges();
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
          {#each items as { change, tweak, from } (change.tweakId)}
            <li class="flex items-center gap-2 rounded px-2 py-1.5 hover:bg-muted" transition:expand>
              <button
                type="button"
                class="min-w-0 flex-1 cursor-pointer truncate text-left text-ui"
                onclick={() => tweakDetailsModalStore.open(change.tweakId)}
              >
                <span class="text-foreground">{tweak?.definition.name ?? change.tweakId}</span>
                <span class="text-foreground-muted"> {from ? `${from} → ` : "→ "}{change.optionLabel}</span>
              </button>
              <button
                type="button"
                class="flex h-6 w-6 shrink-0 cursor-pointer items-center justify-center rounded text-foreground-muted hover:bg-muted hover:text-foreground"
                aria-label="Unstage {tweak?.definition.name ?? change.tweakId}"
                onclick={() => pendingChangesStore.remove(change.tweakId)}
              >
                <Icon icon="mdi:close" width="14" />
              </button>
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
              {count === 1 ? "1 change" : `${count} changes`} ready to apply
            </span>
            {#if needsReboot}
              <span class="block truncate text-xs text-foreground-muted">Some need a restart to take effect</span>
            {/if}
          </span>
          <Icon
            icon="mdi:chevron-up"
            width="16"
            class="shrink-0 text-foreground-muted transition-transform duration-normal {expanded ? 'rotate-180' : ''}"
          />
        </button>

        <div class="ml-auto flex shrink-0 items-center gap-2">
          <button
            type="button"
            class="h-8 cursor-pointer rounded-md border border-border bg-secondary px-3 text-ui font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            disabled={busy}
            onclick={() => pendingChangesStore.clear()}
          >
            Discard
          </button>
          <button
            type="button"
            class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md bg-accent px-3.5 text-ui font-semibold text-accent-foreground hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-60"
            disabled={busy}
            onclick={requestApply}
          >
            {#if applying}<Icon icon="mdi:loading" width="14" class="animate-spin" />{/if}
            Apply
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<Modal open={reviewOpen} onclose={() => (reviewing = false)} size="lg" labelledBy="apply-review-title">
  <ModalHeader>
    <div class="flex items-center gap-3">
      <Icon icon="mdi:alert" width="22" class="shrink-0 text-warning" />
      <div>
        <h2 id="apply-review-title" class="m-0 text-base font-semibold text-foreground">
          Review {count === 1 ? "1 change" : `${count} changes`}
        </h2>
        <p class="m-0 mt-0.5 text-ui text-foreground-muted">
          {highRiskCount === 1 ? "1 change is" : `${highRiskCount} changes are`} high risk. Check them before applying.
        </p>
      </div>
    </div>
  </ModalHeader>
  <ModalBody>
    <ul class="m-0 list-none space-y-1.5 p-0">
      {#each items as { change, tweak, from, highRisk } (change.tweakId)}
        <li class="rounded-md border px-3 py-2 text-ui {highRisk ? 'border-error/35 bg-error/6' : 'border-border'}">
          <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
            <span class="font-medium text-foreground">{tweak?.definition.name ?? change.tweakId}</span>
            <span class="text-foreground-muted">
              {from ? `${from} → ` : "→ "}<span class="font-medium text-foreground">{change.optionLabel}</span>
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
