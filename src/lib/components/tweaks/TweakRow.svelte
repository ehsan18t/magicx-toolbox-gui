<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { SegmentedSwitch, Select } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { searchStore } from "$lib/stores/search.svelte";
  import { openTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import {
    errorStore,
    keepCurrentState,
    loadingStore,
    pendingChangesStore,
    revertTweak,
    stageChange,
    unstageChange,
  } from "$lib/stores/tweaks.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { attentionCause, permissionInfoFor, RISK_INFO } from "$lib/types";
  import { availabilityLabel, RISK_TONE, stateSummary, TONE_TEXT } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";

  interface Props {
    tweak: TweakWithStatus;
    titleSlot?: Snippet;
    descriptionSlot?: Snippet;
    /** Extra meta-line content, e.g. the category in search results. */
    context?: Snippet;
  }

  let { tweak, titleSlot, descriptionSlot, context }: Props = $props();

  const STACK_BELOW_ROW_WIDTH = 520;
  // Sentinel dropdown value for the computed "System Default" position (ADR-0003).
  const SYSTEM_DEFAULT = "__system_default__";

  let rowWidth = $state(800);
  const stacked = $derived(rowWidth < STACK_BELOW_ROW_WIDTH);

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const availability = $derived(def.availability);
  const isLoading = $derived(loadingStore.isLoading(def.id));
  const tweakError = $derived(errorStore.getError(def.id));
  const isSelected = $derived(tweakDetailsModalStore.tweakId === def.id);
  const summary = $derived(stateSummary(status));

  const unknownTip = $derived.by(() => {
    if (status.state !== "unknown") return "";
    const causes = status.unknownReasons.map((r) => `${r.effect}: ${r.cause}`).join("; ");
    const base = `Could not read this tweak's state (${causes || "unknown"}).`;
    return status.needsElevation ? `${base} Restart as administrator to resolve.` : base;
  });

  const controlDisabledReason = $derived.by(() => {
    if (status.state === "unavailable") return status.unavailableReason ?? "Not available on this system";
    if (availability.state !== "available") return availability.reason;
    return null;
  });
  const controlDisabled = $derived(isLoading || controlDisabledReason !== null);

  let isHighlighting = $state(false);
  $effect(() => {
    if (searchStore.highlightTweakId !== def.id) return;
    isHighlighting = true;
    const timer = setTimeout(() => {
      isHighlighting = false;
      searchStore.clearHighlight();
    }, 1500);
    return () => clearTimeout(timer);
  });

  let showConfirmDialog = $state(false);
  let showRestoreConfirmDialog = $state(false);
  let showKeepStateConfirmDialog = $state(false);

  const riskInfo = $derived(RISK_INFO[def.risk_level]);
  const isHighRisk = $derived(def.risk_level === "high" || def.risk_level === "critical");
  const permissionInfo = $derived(permissionInfoFor(def.required_level));
  const isFavorite = $derived(favoritesStore.isFavorite(def.id));
  const hasSnapshot = $derived(status.has_backup);

  // Needs Attention (ADR-0001/0002): the engine's own record, kept per tweak.
  const attention = $derived(status.attention);
  const restoreFailed = $derived(attention?.reason === "restore_failed");
  const attentionDetail = $derived(
    (attention?.items ?? []).map((item) => (/[.!?]$/.test(item.message) ? item.message : `${item.message}.`)).join(" "),
  );

  const pendingChange = $derived(pendingChangesStore.get(def.id));
  const hasPending = $derived(pendingChange !== undefined);
  const activeOption = $derived(status.activeOption);

  const optionLabels = $derived(def.optionLabels);
  const isSegmented = $derived(optionLabels.length <= 2);

  // ADR-0003: offered only while it is the detected state. Not `activeOption == null`: Unknown and
  // loading also have no active option, and must show nothing selected.
  const atSystemDefault = $derived(status.state === "system_default");

  const LONG_SEGMENT_LABELS = 34;

  const segments = $derived.by(() => {
    const [first, second] = optionLabels;
    const ordered = second === undefined ? [first] : [first, second];
    if (atSystemDefault) ordered.splice(second === undefined ? 0 : 1, 0, SYSTEM_DEFAULT);
    return ordered.map((label, i) => {
      const isDefault = label === SYSTEM_DEFAULT;
      const unavailable = isDefault ? undefined : status.unavailableOptions.find((u) => u.label === label);
      return {
        value: i,
        label: isDefault ? "System default" : unavailable ? `${label} (unavailable)` : label,
        icon: isDefault ? "mdi:monitor" : undefined,
        disabled: !!unavailable,
        target: label,
      };
    });
  });

  const stackControl = $derived(
    stacked || (isSegmented && segments.reduce((n, s) => n + s.label.length, 0) > LONG_SEGMENT_LABELS),
  );

  const selectValue = $derived(pendingChange?.optionLabel ?? activeOption ?? (atSystemDefault ? SYSTEM_DEFAULT : null));
  // -1 selects no segment (Unknown / loading).
  const segmentValue = $derived(segments.findIndex((s) => s.target === selectValue));
  const selectOptions = $derived.by(() => {
    const opts: { value: string; label: string; disabled?: boolean }[] = [];
    if (atSystemDefault) opts.push({ value: SYSTEM_DEFAULT, label: "System default" });
    for (const label of optionLabels) {
      const un = status.unavailableOptions.find((u) => u.label === label);
      opts.push({ value: label, label: un ? `${label} (unavailable)` : label, disabled: !!un });
    }
    return opts;
  });

  let pendingHighRiskLabel: string | null = $state(null);

  // System Default is a Restore, never an Apply (ADR-0003), and only from an authored option:
  // clicking it while already there must only unstage.
  function selectTarget(target: string) {
    if (target === SYSTEM_DEFAULT) {
      unstageChange(def.id);
      if (activeOption && hasSnapshot) handleRestoreClick();
      return;
    }
    if (target === activeOption) {
      unstageChange(def.id);
      return;
    }
    if (isHighRisk) {
      pendingHighRiskLabel = target;
      showConfirmDialog = true;
      return;
    }
    stageChange(def.id, { tweakId: def.id, optionLabel: target });
  }

  function handleConfirmHighRisk() {
    showConfirmDialog = false;
    if (pendingHighRiskLabel !== null) {
      stageChange(def.id, { tweakId: def.id, optionLabel: pendingHighRiskLabel });
      pendingHighRiskLabel = null;
    }
  }

  function handleRestoreClick() {
    if (isHighRisk) showRestoreConfirmDialog = true;
    else void executeRestore();
  }

  async function executeRestore() {
    showRestoreConfirmDialog = false;
    await revertTweak(def.id, { showToast: true, tweakName: def.name });
  }

  async function executeKeepCurrentState() {
    showKeepStateConfirmDialog = false;
    await keepCurrentState(def.id, { showToast: true, tweakName: def.name });
  }

  function handleRowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest("button, a, input, [role='radiogroup'], [role='combobox'], [role='listbox']")) return;
    openTweakDetailsModal(def.id);
  }

  const restoreDisabled = $derived(isLoading || availability.state !== "available");
  const restoreTip = $derived(
    availability.state !== "available"
      ? availability.reason
      : restoreFailed
        ? "Retry restoring the original state"
        : "Restore the original state from the snapshot",
  );
  const stripe = $derived(
    attention ? "bg-error" : hasPending ? "bg-warning" : status.is_applied ? "bg-accent" : "bg-transparent",
  );
</script>

{#snippet metaItem(icon: string, label: string, tone: string, tip?: string | null, spin?: boolean)}
  <span class="inline-flex max-w-full items-center gap-1 {tone}" use:tooltip={tip ?? null}>
    <Icon {icon} width="13" class="shrink-0 {spin ? 'animate-spin' : ''}" />
    <span class="truncate">{label}</span>
  </span>
{/snippet}

{#snippet actionButton(icon: string, label: string, onclick: () => void, tip: string, disabled = false)}
  <button
    type="button"
    class="inline-flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-md text-xs font-medium text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50 {stacked
      ? 'w-7 justify-center'
      : 'px-2'}"
    aria-label={label}
    use:tooltip={tip}
    {disabled}
    {onclick}
  >
    <Icon {icon} width="15" class="shrink-0" />
    {#if !stacked}<span>{label}</span>{/if}
  </button>
{/snippet}

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<article
  id="tweak-{def.id}"
  bind:clientWidth={rowWidth}
  class="relative cursor-pointer overflow-hidden rounded-lg border bg-card transition-colors duration-150 {isSelected
    ? 'border-accent/70'
    : hasPending
      ? 'border-warning/45'
      : 'border-border hover:border-border-hover'} {isHighlighting ? 'tweak-highlight' : ''}"
  onclick={handleRowClick}
>
  <span class="absolute top-3 bottom-3 left-0 w-0.75 rounded-r-full {stripe}" aria-hidden="true"></span>

  <div class="flex flex-col gap-2.5 py-3 pr-3 pl-4">
    <div class="flex gap-x-6 gap-y-2.5 {stackControl ? 'flex-col' : 'items-start'}">
      <div class="min-w-0 flex-1">
        <h3 class="m-0 text-sm leading-snug font-semibold wrap-break-word text-foreground">
          {#if titleSlot}{@render titleSlot()}{:else}{def.name}{/if}
        </h3>
        <p class="m-0 mt-0.5 text-[13px] leading-snug text-foreground-muted">
          {#if descriptionSlot}{@render descriptionSlot()}{:else}{def.description}{/if}
        </p>
      </div>

      <div
        class="min-w-0 {stackControl
          ? 'w-full'
          : isSegmented
            ? 'max-w-[45%] shrink-0'
            : 'w-fit max-w-[45%] min-w-44 shrink-0'}"
        use:tooltip={controlDisabledReason}
      >
        {#if isSegmented}
          <SegmentedSwitch
            value={segmentValue}
            options={segments}
            pending={hasPending}
            loading={isLoading}
            disabled={controlDisabled}
            stretch={stackControl}
            onchange={(i) => {
              const t = segments[i]?.target;
              if (t !== undefined) selectTarget(t);
            }}
          />
        {:else}
          <Select
            value={selectValue}
            options={selectOptions}
            placeholder={status.state === "unknown" ? "Unknown" : status.state === "loading" ? "Checking…" : undefined}
            pending={hasPending}
            loading={isLoading}
            disabled={controlDisabled}
            onchange={(v) => selectTarget(String(v))}
          />
        {/if}
      </div>
    </div>

    {#if def.warning}
      <div class="flex gap-2 rounded-md bg-warning/8 px-2.5 py-2 text-xs leading-relaxed text-foreground">
        <Icon icon="mdi:alert" width="14" class="mt-px shrink-0 text-warning" />
        <span class="min-w-0">{def.warning}</span>
      </div>
    {/if}

    {#if attention}
      <div class="rounded-md border border-error/30 bg-error/8 px-2.5 py-2 text-xs leading-relaxed">
        <div class="flex gap-2">
          <Icon icon="mdi:alert-circle" width="14" class="mt-px shrink-0 text-error" />
          <span class="min-w-0">
            <span class="font-semibold text-error">Needs attention.</span>
            {attentionCause(attention.reason)}.
            {#if attentionDetail}<span class="text-foreground-muted">{attentionDetail}</span>{/if}
            <span class="text-foreground-muted">
              {hasSnapshot
                ? "Your snapshot is safe: restore it, or keep things as they are."
                : "No snapshot is left to restore, so you can only keep the current state."}
            </span>
          </span>
        </div>
        <div class="mt-2 flex flex-wrap gap-2 pl-5.5">
          {#if hasSnapshot}
            <button
              type="button"
              class="h-7 cursor-pointer rounded-md border border-border bg-secondary px-2.5 font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
              onclick={handleRestoreClick}
              disabled={restoreDisabled}
              use:tooltip={restoreTip}
            >
              {restoreFailed ? "Retry restore" : "Restore"}
            </button>
          {/if}
          <button
            type="button"
            class="h-7 cursor-pointer rounded-md border border-border bg-secondary px-2.5 font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            onclick={() => (showKeepStateConfirmDialog = true)}
            disabled={isLoading}
          >
            Keep current state
          </button>
        </div>
      </div>
    {/if}

    {#if tweakError}
      <div class="flex items-start gap-2 rounded-md border border-error/30 bg-error/8 px-2.5 py-2 text-xs text-error">
        <Icon icon="mdi:alert-circle" width="14" class="mt-px shrink-0" />
        <span class="min-w-0 flex-1 wrap-break-word">{tweakError}</span>
        <button
          type="button"
          class="flex shrink-0 cursor-pointer rounded p-0.5 text-error/70 hover:bg-error/10 hover:text-error"
          onclick={() => errorStore.clearError(def.id)}
          aria-label="Dismiss error"
        >
          <Icon icon="mdi:close" width="14" />
        </button>
      </div>
    {/if}

    <div class="flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
      {@render metaItem(
        summary.icon,
        summary.label,
        TONE_TEXT[summary.tone],
        status.state === "unknown"
          ? unknownTip
          : status.state === "unavailable"
            ? (status.unavailableReason ?? "Not available on this system")
            : "Current state",
        status.state === "loading",
      )}
      {#if pendingChange}
        {@render metaItem(
          "mdi:arrow-right",
          `${pendingChange.optionLabel} pending`,
          "text-warning",
          "Staged, not applied yet",
        )}
      {/if}
      {@render metaItem(
        "mdi:shield-half-full",
        `${riskInfo.name} risk`,
        TONE_TEXT[RISK_TONE[def.risk_level]],
        riskInfo.description,
      )}
      {#if permissionInfo}
        {@render metaItem(
          permissionInfo.icon,
          permissionInfo.name,
          "text-foreground-muted",
          permissionInfo.description,
        )}
      {/if}
      {#if def.requires_reboot}
        {@render metaItem("mdi:restart", "Restart", "text-info", "Restart required after applying or restoring")}
      {/if}
      {#if availability.state !== "available" && status.state !== "unavailable"}
        {@render metaItem(
          "mdi:shield-lock-outline",
          availabilityLabel(availability),
          "text-warning",
          availability.reason,
        )}
      {/if}
      {#if status.residues.length > 0}
        {@render metaItem(
          "mdi:information-outline",
          "Residue",
          "text-info",
          `Residual settings remain outside the active option: ${status.residues.join(", ")}`,
        )}
      {/if}
      {#if status.heldShared.length > 0}
        {@render metaItem(
          "mdi:link-variant",
          "Shared",
          "text-foreground-muted",
          `Shared with: ${status.heldShared.map((h) => `${h.shared} (${h.holders.join(", ")})`).join("; ")}`,
        )}
      {/if}
      {#if context}{@render context()}{/if}

      <div class="ml-auto flex items-center gap-0.5">
        {#if hasSnapshot && !attention}
          {@render actionButton("mdi:history", "Restore", handleRestoreClick, restoreTip, restoreDisabled)}
        {/if}
        <button
          type="button"
          class="inline-flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md hover:bg-muted {isFavorite
            ? 'text-warning'
            : 'text-foreground-muted hover:text-foreground'}"
          onclick={() => favoritesStore.toggle(def.id)}
          aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
          aria-pressed={isFavorite}
          use:tooltip={isFavorite ? "Remove from favorites" : "Add to favorites"}
        >
          <Icon icon={isFavorite ? "mdi:star" : "mdi:star-outline"} width="16" />
        </button>
        {@render actionButton("mdi:chevron-right", "Details", () => openTweakDetailsModal(def.id), "Open details")}
      </div>
    </div>
  </div>
</article>

<ConfirmDialog
  open={showConfirmDialog}
  title="Apply High-Risk Tweak?"
  message="This tweak is marked as {def.risk_level} risk. {riskInfo.description} Are you sure you want to apply it?"
  confirmText="Yes, Apply"
  cancelText="Cancel"
  onconfirm={handleConfirmHighRisk}
  oncancel={() => {
    showConfirmDialog = false;
    pendingHighRiskLabel = null;
  }}
/>

<ConfirmDialog
  open={showRestoreConfirmDialog}
  title="Restore Snapshot?"
  message="This will restore the original state from before the tweak was applied."
  confirmText="Restore"
  cancelText="Cancel"
  onconfirm={executeRestore}
  oncancel={() => (showRestoreConfirmDialog = false)}
/>

<ConfirmDialog
  open={showKeepStateConfirmDialog}
  title="Keep Current State?"
  message="This accepts the current state as-is and releases any saved snapshot. The original state can no longer be restored for this tweak."
  confirmText="Keep current state"
  cancelText="Cancel"
  onconfirm={executeKeepCurrentState}
  oncancel={() => (showKeepStateConfirmDialog = false)}
/>
