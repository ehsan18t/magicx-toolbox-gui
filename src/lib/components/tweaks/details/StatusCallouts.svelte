<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, Callout, ICON_SIZE } from "$lib/components/ui";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import type { AttentionItem, TweakWithStatus } from "$lib/types";
  import { attentionCause, availabilityTitle } from "$lib/utils/tweakPresentation";
  import RestoreButton from "./RestoreButton.svelte";

  let { tweak }: { tweak: TweakWithStatus } = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);

  let keeping = $state(false);

  function attentionHint(item: AttentionItem): string {
    if (item.kind === "no_undo") return "(this one cannot be retried)";
    if (item.class === "busy") return "(retrying later may succeed)";
    if (item.class === "access_denied" && elevationStore.level === "User") return "(restart as administrator to retry)";
    return "";
  }

  const holderName = (id: string) => tweaksStore.tweak(id)?.definition.name ?? id;

  async function keepCurrent() {
    keeping = true;
    try {
      await tweakActionsStore.keepWithConfirm(def);
    } finally {
      keeping = false;
    }
  }
</script>

{#if def.warning}
  <Callout tone="warning" density="panel" icon="mdi:alert">
    <p class="m-0 text-ui leading-relaxed text-foreground">{def.warning}</p>
  </Callout>
{/if}

{#if def.availability.state !== "available"}
  <Callout tone="warning" density="panel" icon="mdi:shield-lock-outline">
    <p class="m-0 text-ui leading-relaxed">
      <span class="font-semibold">{availabilityTitle(def.availability)}.</span>
      <span class="text-foreground-muted">{def.availability.reason}</span>
    </p>
  </Callout>
{/if}

{#if status.state === "unavailable" && status.unavailableReason}
  <Callout density="panel" icon="mdi:cancel" class="bg-card">
    <p class="m-0 text-ui text-foreground-muted">{status.unavailableReason}</p>
  </Callout>
{/if}

{#if status.state === "unknown" && status.unknownReasons.length > 0}
  <Callout tone="warning" density="panel">
    <p class="m-0 mb-1.5 flex items-center gap-2 text-ui font-semibold">
      <Icon icon="mdi:help-circle-outline" width={ICON_SIZE.md} class="text-warning" />
      Could not determine state
    </p>
    <ul class="m-0 list-none space-y-1 p-0 pl-6">
      {#each status.unknownReasons as reason, i (`${reason.effect}-${i}`)}
        <li class="text-xs text-foreground-muted">
          <span class="font-mono break-all text-foreground">{reason.effect}</span>
          ({reason.cause}{reason.needs_elevation ? ", restart as admin to resolve" : ""})
        </li>
      {/each}
    </ul>
  </Callout>
{/if}

{#if status.attention}
  <Callout tone="error" density="panel">
    <p class="m-0 flex gap-2 text-ui leading-relaxed">
      <Icon icon="mdi:alert-circle" width={ICON_SIZE.lg} class="mt-0.5 shrink-0 text-error" />
      <span>
        <span class="font-semibold">Needs attention.</span>
        <span class="text-foreground-muted">
          {attentionCause(status.attention.reason)}{status.hasHistory
            ? ", so the snapshot was kept."
            : ". There is no snapshot left to restore."}
        </span>
      </span>
    </p>
    {#if status.attention.items.length}
      <ul class="m-0 mt-2 list-none space-y-1 p-0 pl-6.5">
        {#each status.attention.items as item, i (`${item.effect}-${i}`)}
          <li class="text-xs text-foreground-muted">
            {#if item.effect}<span class="font-mono break-all text-foreground">{item.effect}</span>:{/if}
            {item.message}
            {attentionHint(item)}
          </li>
        {/each}
      </ul>
    {/if}
    <div class="mt-3 ml-6.5 flex flex-wrap gap-2">
      {#if status.hasHistory}<RestoreButton {tweak} />{/if}
      <!-- Consent stays reachable whenever a record exists, entries left or not (ADR-0002). -->
      <Button loading={keeping} disabled={tweakActionsStore.isRunning(def.id)} onclick={keepCurrent}>
        {#if !keeping}<Icon icon="mdi:check" width={ICON_SIZE.md} />{/if}
        Keep current state
      </Button>
    </div>
  </Callout>
{/if}

{#if status.residues.length > 0 || status.heldShared.length > 0}
  <Callout density="panel" class="space-y-2 bg-card text-ui text-foreground-muted">
    {#if status.residues.length > 0}
      <p class="m-0 flex gap-2">
        <Icon icon="mdi:information-outline" width={ICON_SIZE.md} class="mt-0.5 shrink-0 text-info" />
        <span>Residual settings remain outside the active option: {status.residues.join(", ")}</span>
      </p>
    {/if}
    {#if status.heldShared.length > 0}
      <p class="m-0 flex gap-2">
        <Icon icon="mdi:link-variant" width={ICON_SIZE.md} class="mt-0.5 shrink-0" />
        <span>
          Shared settings held: {status.heldShared
            .map((h) => `${h.shared} (${h.holders.map(holderName).join(", ")})`)
            .join("; ")}
        </span>
      </p>
    {/if}
  </Callout>
{/if}
