<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { HighlightedText, TONE_TEXT } from "$lib/components/ui";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { expand } from "$lib/utils/motion";
  import {
    attentionCause,
    availabilityLabel,
    isHighRisk,
    permissionInfoFor,
    RISK_INFO,
    RISK_TONE,
    stateSummary,
    usesDropdown,
  } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import { searchHighlight } from "./searchHighlight.svelte";
  import TweakControl from "./TweakControl.svelte";

  interface Props {
    tweak: TweakWithStatus;
    titleSlot?: Snippet;
    descriptionSlot?: Snippet;
    /** Extra meta-line content, e.g. the category in search results. */
    context?: Snippet;
  }

  let { tweak, titleSlot, descriptionSlot, context }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const availability = $derived(def.availability);
  const isLoading = $derived(tweakActionsStore.isRunning(def.id));
  const tweakError = $derived(tweakActionsStore.error(def.id));
  const isSelected = $derived(tweakDetailsModalStore.tweakId === def.id);
  const summary = $derived(stateSummary(status));
  const filterMatch = $derived(titleSlot ? null : pageFilterStore.match(def.id));

  const unknownTip = $derived.by(() => {
    if (status.state !== "unknown") return "";
    const causes = status.unknownReasons.map((r) => `${r.effect}: ${r.cause}`).join("; ");
    const base = `Could not read this tweak's state (${causes || "unknown"}).`;
    return status.needsElevation ? `${base} Restart as administrator to resolve.` : base;
  });

  let rowEl = $state<HTMLElement | null>(null);
  const highlight = searchHighlight(
    () => def.id,
    () => rowEl,
  );

  const riskInfo = $derived(RISK_INFO[def.riskLevel]);
  const highRisk = $derived(isHighRisk(def.riskLevel));
  const permissionInfo = $derived(permissionInfoFor(def.requiredLevel));
  const isFavorite = $derived(favoritesStore.isFavorite(def.id));
  const hasSnapshot = $derived(status.hasHistory);

  // Needs Attention (ADR-0001/0002): the engine's own record, kept per tweak.
  const attention = $derived(status.attention);
  const restoreFailed = $derived(attention?.reason === "restore_failed");
  const attentionDetail = $derived(
    (attention?.items ?? []).map((item) => (/[.!?]$/.test(item.message) ? item.message : `${item.message}.`)).join(" "),
  );

  const pendingChange = $derived(pendingChangesStore.change(def.id));
  const hasPending = $derived(pendingChange !== undefined);
  // Hidden until asked for, but staging a change is when it matters, so that opens it too.
  let warningToggled = $state(false);
  const warningOpen = $derived(warningToggled || hasPending);
  const dropdown = $derived(usesDropdown(def));

  const handleRestoreClick = () => tweakActionsStore.restoreWithConfirm(def, highRisk);
  const handleKeepCurrentState = () => tweakActionsStore.keepWithConfirm(def);

  function handleRowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest("button, a, input, [role='radiogroup'], [role='combobox'], [role='listbox']")) return;
    tweakDetailsModalStore.open(def.id);
  }

  const restoreDisabled = $derived(isLoading || availability.state !== "available");
  const restoreTip = $derived(
    availability.state !== "available"
      ? availability.reason
      : restoreFailed
        ? "Retry restoring the saved state"
        : "Restore the state saved before the last change",
  );
  const stripe = $derived(
    attention ? "bg-error" : hasPending ? "bg-warning" : status.state === "active" ? "bg-accent" : "bg-transparent",
  );
</script>

{#snippet marked(text: string, ranges: number[])}
  <HighlightedText {text} {ranges} />
{/snippet}

{#snippet metaItem(icon: string, label: string, tone: string, tip?: string | null, spin?: boolean)}
  <span class="inline-flex max-w-full items-center gap-1 {tone}" use:tooltip={tip ?? null}>
    <Icon {icon} width="13" class="shrink-0 {spin ? 'animate-spin' : ''}" />
    <span class="truncate">{label}</span>
  </span>
{/snippet}

{#snippet actionButton(icon: string, label: string, onclick: () => void, tip: string, disabled = false)}
  <button
    type="button"
    class="inline-flex h-7 shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-md px-2 text-xs font-medium text-foreground-muted hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50 @max-item-row:w-7 @max-item-row:px-0"
    aria-label={label}
    use:tooltip={tip}
    {disabled}
    {onclick}
  >
    <Icon {icon} width="15" class="shrink-0" />
    <span class="@max-item-row:hidden">{label}</span>
  </button>
{/snippet}

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<article
  id="tweak-{def.id}"
  bind:this={rowEl}
  class="@container relative flex cursor-pointer flex-col overflow-hidden rounded-lg border bg-card transition-colors {isSelected
    ? 'border-accent/70'
    : hasPending
      ? 'border-warning/45'
      : 'border-border hover:border-border-hover'} {highlight.active ? 'animate-highlight' : ''}"
  onclick={handleRowClick}
>
  <span class="absolute top-3 bottom-3 left-0 w-0.75 rounded-r-full transition-colors {stripe}" aria-hidden="true"
  ></span>

  <div class="flex flex-1 flex-col gap-2.5 py-3 pr-3 pl-4">
    <div class="grid grid-cols-item-row items-center gap-x-6 gap-y-1 @max-item-row:grid-cols-1 @max-item-row:gap-y-2">
      <h3 class="m-0 text-sm leading-snug font-semibold wrap-break-word text-foreground">
        {#if titleSlot}{@render titleSlot()}{:else if filterMatch}{@render marked(
            def.name,
            filterMatch.nameRanges,
          )}{:else}{def.name}{/if}
      </h3>

      <TweakControl
        {tweak}
        class="max-w-item-row-control @max-item-row:max-w-full {dropdown ? '@max-item-row:w-full' : ''}"
      />

      <p class="col-span-full m-0 text-ui leading-snug text-foreground-muted">
        {#if descriptionSlot}{@render descriptionSlot()}{:else if filterMatch}{@render marked(
            def.description,
            filterMatch.descriptionRanges,
          )}{:else}{def.description}{/if}
      </p>
    </div>

    {#if def.warning && warningOpen}
      <div
        id="warning-{def.id}"
        transition:expand
        class="flex gap-2 rounded-md bg-warning/8 px-2.5 py-2 text-xs leading-relaxed text-foreground"
      >
        <Icon icon="mdi:alert" width="14" class="mt-px shrink-0 text-warning" />
        <span class="min-w-0">{def.warning}</span>
      </div>
    {/if}

    {#if attention}
      <div class="rounded-md border border-error/30 bg-error/8 px-2.5 py-2 text-xs leading-relaxed" transition:expand>
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
            onclick={handleKeepCurrentState}
            disabled={isLoading}
          >
            Keep current state
          </button>
        </div>
      </div>
    {/if}

    {#if tweakError}
      <div
        class="flex items-start gap-2 rounded-md border border-error/30 bg-error/8 px-2.5 py-2 text-xs text-error"
        transition:expand
      >
        <Icon icon="mdi:alert-circle" width="14" class="mt-px shrink-0" />
        <span class="min-w-0 flex-1 wrap-break-word">{tweakError}</span>
        <button
          type="button"
          class="flex shrink-0 cursor-pointer rounded p-0.5 text-error/70 hover:bg-error/10 hover:text-error"
          onclick={() => tweakActionsStore.clearError(def.id)}
          aria-label="Dismiss error"
        >
          <Icon icon="mdi:close" width="14" />
        </button>
      </div>
    {/if}

    <div class="mt-auto flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
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
        <span class="inline-flex max-w-full animate-pop-in items-center gap-1 text-warning">
          {@render metaItem("mdi:arrow-right", `${pendingChange.optionLabel} pending`, "", "Staged, not applied yet")}
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
      {@render metaItem(
        "mdi:shield-half-full",
        `${riskInfo.name} risk`,
        TONE_TEXT[RISK_TONE[def.riskLevel]],
        riskInfo.description,
      )}
      {#if def.warning}
        <button
          type="button"
          class="inline-flex cursor-pointer items-center gap-1 rounded-full border border-warning/35 bg-warning/10 py-0.5 pr-1 pl-2 font-medium text-warning hover:bg-warning/20 disabled:cursor-default disabled:opacity-70 disabled:hover:bg-warning/10"
          aria-expanded={warningOpen}
          aria-controls={warningOpen ? `warning-${def.id}` : undefined}
          disabled={hasPending}
          use:tooltip={hasPending ? "Shown while a change is staged" : warningOpen ? "Hide warning" : "Show warning"}
          onclick={() => (warningToggled = !warningToggled)}
        >
          <Icon icon="mdi:alert" width="13" class="shrink-0" />
          Warning
          <Icon
            icon="mdi:chevron-down"
            width="14"
            class="shrink-0 transition-transform duration-normal {warningOpen ? 'rotate-180' : ''}"
          />
        </button>
      {/if}
      {#if permissionInfo}
        {@render metaItem(
          permissionInfo.icon,
          permissionInfo.name,
          "text-foreground-muted",
          permissionInfo.description,
        )}
      {/if}
      {#if def.requiresReboot}
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
        {@render actionButton(
          "mdi:chevron-right",
          "Details",
          () => tweakDetailsModalStore.open(def.id),
          "Open details",
        )}
      </div>
    </div>
  </div>
</article>
