<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Button, Callout } from "$lib/components/ui";
  import type { SearchResult } from "$lib/stores/search.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { expand, pop } from "$lib/utils/motion";
  import {
    attentionCause,
    availabilityLabel,
    ensureSentence,
    isHighRisk,
    permissionInfoFor,
    restoreState,
    RISK_INFO,
    RISK_TONE,
    rowDomId,
    stateSummary,
    stateTip,
    usesDropdown,
  } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import FavoriteButton from "./FavoriteButton.svelte";
  import ItemRow, { type RowStripe } from "./ItemRow.svelte";
  import MetaItem from "./MetaItem.svelte";
  import RowAction from "./RowAction.svelte";
  import TweakControl from "./TweakControl.svelte";
  import WarningNotice from "./WarningNotice.svelte";
  import WarningToggle from "./WarningToggle.svelte";

  interface Props {
    tweak: TweakWithStatus;
    match?: SearchResult;
    context?: Snippet;
  }

  let { tweak, match, context }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const availability = $derived(def.availability);
  const isLoading = $derived(tweakActionsStore.isRunning(def.id));
  const summary = $derived(stateSummary(status));
  const riskInfo = $derived(RISK_INFO[def.riskLevel]);
  const permissionInfo = $derived(permissionInfoFor(def.requiredLevel));
  const hasSnapshot = $derived(status.hasHistory);
  const restore = $derived(restoreState(def, status, isLoading));
  const warningId = $derived(`${rowDomId("tweak", def.id)}-warning`);

  // Needs Attention (ADR-0001/0002): the engine's own record, kept per tweak.
  const attention = $derived(status.attention);
  const restoreFailed = $derived(attention?.reason === "restore_failed");
  const attentionDetail = $derived((attention?.items ?? []).map((item) => ensureSentence(item.message)).join(" "));

  const pendingChange = $derived(pendingChangesStore.change(def.id));
  const hasPending = $derived(pendingChange !== undefined);
  // Hidden until asked for, but staging a change is when it matters, so that opens it too.
  let warningToggled = $state(false);
  const warningOpen = $derived(warningToggled || hasPending);

  const stripe = $derived.by((): RowStripe | null => {
    if (attention) return "error";
    if (hasPending) return "warning";
    return status.state === "active" ? "accent" : null;
  });

  const openDetails = () => tweakDetailsModalStore.open(def.id);
  const handleRestore = () => tweakActionsStore.restoreWithConfirm(def, isHighRisk(def.riskLevel));

  function handleRowClick(e: MouseEvent) {
    const interactive = "button, a, input, [role='radiogroup'], [role='combobox'], [role='listbox']";
    if (e.target instanceof Element && e.target.closest(interactive)) return;
    openDetails();
  }
</script>

<ItemRow
  kind="tweak"
  id={def.id}
  title={def.name}
  description={def.description}
  {match}
  {stripe}
  emphasis={tweakDetailsModalStore.tweakId === def.id ? "selected" : hasPending ? "pending" : "none"}
  error={tweakActionsStore.error(def.id)}
  ondismisserror={() => tweakActionsStore.clearError(def.id)}
  {context}
  onclick={handleRowClick}
>
  {#snippet control()}
    <TweakControl
      {tweak}
      class="max-w-item-row-control @max-item-row:max-w-full {usesDropdown(def) ? '@max-item-row:w-full' : ''}"
    />
  {/snippet}

  {#snippet notices()}
    <WarningNotice id={warningId} text={def.warning} open={warningOpen} />

    {#if attention}
      <div transition:expand>
        <Callout tone="error" density="compact" icon="mdi:alert-circle" class="leading-relaxed">
          <div class="min-w-0 flex-1">
            <span class="font-semibold text-error">Needs attention.</span>
            {attentionCause(attention.reason)}.
            {#if attentionDetail}<span class="text-foreground-muted">{attentionDetail}</span>{/if}
            <span class="text-foreground-muted">
              {hasSnapshot
                ? "Your snapshot is safe: restore it, or keep things as they are."
                : "No snapshot is left to restore, so you can only keep the current state."}
            </span>
            <div class="mt-2 flex flex-wrap gap-2">
              {#if hasSnapshot}
                <span class="flex" use:tooltip={restore.tip}>
                  <Button size="sm" class="leading-relaxed" disabled={restore.disabled} onclick={handleRestore}>
                    {restoreFailed ? "Retry restore" : "Restore"}
                  </Button>
                </span>
              {/if}
              <Button
                size="sm"
                class="leading-relaxed"
                disabled={isLoading}
                onclick={() => tweakActionsStore.keepWithConfirm(def)}
              >
                Keep current state
              </Button>
            </div>
          </div>
        </Callout>
      </div>
    {/if}
  {/snippet}

  {#snippet meta()}
    <MetaItem
      icon={summary.icon}
      label={summary.label}
      tone={summary.tone}
      tooltip={stateTip(status)}
      spin={status.state === "loading"}
      truncate
    />
    {#if pendingChange}
      <!-- in:, not animate-pop-in: a local transition skips the row's own mount, so filtering never replays it. -->
      <span class="inline-flex max-w-full items-center gap-1 text-warning" in:pop>
        <MetaItem
          icon="mdi:arrow-right"
          label="{pendingChange.optionLabel} pending"
          tooltip="Staged, not applied yet"
          truncate
        />
        <button
          type="button"
          class="cursor-pointer rounded px-1 font-medium underline-offset-2 hover:underline"
          aria-label="Undo staged change to {def.name}"
          onclick={() => pendingChangesStore.remove(def.id)}
        >
          Undo
        </button>
      </span>
    {/if}
    <MetaItem
      icon="mdi:shield-half-full"
      label="{riskInfo.name} risk"
      tone={RISK_TONE[def.riskLevel]}
      tooltip={riskInfo.description}
      truncate
    />
    {#if def.warning}
      <WarningToggle
        open={warningOpen}
        controls={warningId}
        lockedReason={hasPending ? "Shown while a change is staged" : null}
        ontoggle={() => (warningToggled = !warningToggled)}
      />
    {/if}
    {#if permissionInfo}
      <MetaItem
        icon={permissionInfo.icon}
        label={permissionInfo.name}
        tone="neutral"
        tooltip={permissionInfo.description}
        truncate
      />
    {/if}
    {#if def.requiresReboot}
      <MetaItem
        icon="mdi:restart"
        label="Restart"
        tone="info"
        tooltip="Restart required after applying or restoring"
        truncate
      />
    {/if}
    {#if availability.state !== "available" && status.state !== "unavailable"}
      <MetaItem
        icon="mdi:shield-lock-outline"
        label={availabilityLabel(availability)}
        tone="warning"
        tooltip={availability.reason}
        truncate
      />
    {/if}
    {#if status.residues.length > 0}
      <MetaItem
        icon="mdi:information-outline"
        label="Residue"
        tone="info"
        tooltip="Residual settings remain outside the active option: {status.residues.join(', ')}"
        truncate
      />
    {/if}
    {#if status.heldShared.length > 0}
      <MetaItem
        icon="mdi:link-variant"
        label="Shared"
        tone="neutral"
        tooltip="Shared with: {status.heldShared.map((h) => `${h.shared} (${h.holders.join(', ')})`).join('; ')}"
        truncate
      />
    {/if}
  {/snippet}

  {#snippet actions()}
    {#if hasSnapshot && !attention}
      <RowAction
        icon="mdi:history"
        label="Restore"
        tooltip={restore.tip}
        disabled={restore.disabled}
        collapses
        onclick={handleRestore}
      />
    {/if}
    <FavoriteButton tweakId={def.id} />
    <RowAction icon="mdi:chevron-right" label="Details" tooltip="Open details" collapses onclick={openDetails} />
  {/snippet}
</ItemRow>
