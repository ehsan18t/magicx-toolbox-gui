<script lang="ts">
  import { LinkButton, MetaItem } from "$lib/components/ui";
  import { tweakDetailsModalStore } from "$lib/stores/detailsModal.svelte";
  import type { SearchResult } from "$lib/stores/search.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { expand, pop } from "$lib/utils/motion";
  import { type MetaFact, pendingFact, rowDomId, tweakMeta, usesDropdown } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import AttentionNotice from "./AttentionNotice.svelte";
  import FavoriteButton from "./FavoriteButton.svelte";
  import ItemRow, { type RowStripe } from "./ItemRow.svelte";
  import RestoreButton from "./RestoreButton.svelte";
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
  const facts = $derived(tweakMeta(tweak, tweaksStore.tweak));
  const warningId = $derived(`${rowDomId("tweak", def.id)}-warning`);

  const pendingChange = $derived(pendingChangesStore.change(def.id));
  const hasPending = $derived(pendingChange !== undefined);
  // Hidden until asked for, but staging a change is when it matters, so that opens it too.
  let warningToggled = $state(false);
  const warningOpen = $derived(warningToggled || hasPending);

  const stripe = $derived.by((): RowStripe | null => {
    if (status.attention) return "error";
    if (hasPending) return "warning";
    return status.state === "active" ? "accent" : null;
  });

  const openDetails = () => tweakDetailsModalStore.open(def.id);

  function handleRowClick(e: MouseEvent) {
    const interactive = "button, a, input, [role='radiogroup'], [role='combobox'], [role='listbox']";
    if (e.target instanceof Element && e.target.closest(interactive)) return;
    openDetails();
  }
</script>

{#snippet fact(item: MetaFact | null)}
  {#if item}<MetaItem {...item} truncate />{/if}
{/snippet}

<ItemRow
  kind="tweak"
  id={def.id}
  title={def.name}
  description={def.description}
  {match}
  {stripe}
  emphasis={tweakDetailsModalStore.openId === def.id ? "selected" : hasPending ? "pending" : "none"}
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
    {#if status.attention}
      <div transition:expand><AttentionNotice {tweak} density="compact" /></div>
    {/if}
  {/snippet}

  {#snippet meta()}
    {@render fact(facts.state)}
    {#if pendingChange}
      <!-- in:, not animate-pop-in: a local transition skips the row's own mount, so filtering never replays it. -->
      <span class="inline-flex max-w-full items-center gap-1 text-warning" in:pop>
        {@render fact(pendingFact(pendingChange.optionLabel))}
        <LinkButton
          variant="hover"
          aria-label="Undo staged change to {def.name}"
          onclick={() => pendingChangesStore.remove(def.id)}
        >
          Undo
        </LinkButton>
      </span>
    {/if}
    {@render fact(facts.risk)}
    {#if def.warning}
      <WarningToggle
        open={warningOpen}
        controls={warningId}
        lockedReason={hasPending ? "Shown while a change is staged" : null}
        ontoggle={() => (warningToggled = !warningToggled)}
      />
    {/if}
    {#if def.requiredLevel !== "User"}{@render fact(facts.permission)}{/if}
    {@render fact(facts.restart)}
    {@render fact(facts.availability)}
    {@render fact(facts.residue)}
    {@render fact(facts.shared)}
  {/snippet}

  {#snippet actions()}
    {#if status.hasHistory && !status.attention}<RestoreButton {tweak} row />{/if}
    <FavoriteButton tweakId={def.id} name={def.name} />
    <RowAction
      icon="mdi:chevron-right"
      label="Details"
      aria-label="Open details for {def.name}"
      tooltip="Open details"
      collapses
      onclick={openDetails}
    />
  {/snippet}
</ItemRow>
