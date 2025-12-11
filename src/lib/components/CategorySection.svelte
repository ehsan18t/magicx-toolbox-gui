<script lang="ts">
  import { applyTweak, loadingStore, revertTweak } from "$lib/stores/tweaks";
  import type { CategoryDefinition, TweakWithStatus } from "$lib/types";
  import { derived } from "svelte/store";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import TweakCard from "./TweakCard.svelte";

  interface Props {
    category: CategoryDefinition;
    tweaks: TweakWithStatus[];
    initialExpanded?: boolean;
  }

  const { category, tweaks, initialExpanded = true }: Props = $props();

  // Use a function to initialize state to avoid state_referenced_locally warning
  const getInitialExpanded = () => initialExpanded;
  let isExpanded = $state(getInitialExpanded());
  let showApplyAllDialog = $state(false);
  let showRevertAllDialog = $state(false);
  let isBatchProcessing = $state(false);

  const appliedCount = $derived(tweaks.filter((t) => t.status.is_applied).length);
  const unappliedTweaks = $derived(tweaks.filter((t) => !t.status.is_applied));
  const appliedTweaks = $derived(tweaks.filter((t) => t.status.is_applied));

  // Check if any tweak in this category is loading
  const isLoading = derived(loadingStore, ($loading) =>
    tweaks.some((t) => $loading.has(t.definition.id)),
  );

  async function handleApplyAll() {
    showApplyAllDialog = false;
    isBatchProcessing = true;

    for (const tweak of unappliedTweaks) {
      await applyTweak(tweak.definition.id);
    }

    isBatchProcessing = false;
  }

  async function handleRevertAll() {
    showRevertAllDialog = false;
    isBatchProcessing = true;

    for (const tweak of appliedTweaks) {
      await revertTweak(tweak.definition.id);
    }

    isBatchProcessing = false;
  }

  function onApplyAllClick(e: MouseEvent) {
    e.stopPropagation();
    if (unappliedTweaks.length > 0) {
      showApplyAllDialog = true;
    }
  }

  function onRevertAllClick(e: MouseEvent) {
    e.stopPropagation();
    if (appliedTweaks.length > 0) {
      showRevertAllDialog = true;
    }
  }
</script>

<section class="mb-4">
  <!-- Header -->
  <div
    class="flex w-full items-center justify-between rounded-lg border border-border bg-card p-0 transition-all duration-200 hover:border-accent/30 hover:bg-accent/5"
  >
    <button
      class="flex flex-1 cursor-pointer items-center border-0 bg-transparent px-4 py-3 text-left"
      onclick={() => (isExpanded = !isExpanded)}
    >
      <div class="flex items-center gap-3">
        <span class="text-2xl">{category.icon}</span>
        <div class="text-left">
          <h2 class="m-0 text-base font-semibold text-foreground">{category.name}</h2>
          <p class="mt-0.5 mb-0 text-xs text-foreground-muted">{category.description}</p>
        </div>
      </div>
    </button>

    <div class="flex items-center gap-3">
      <!-- Batch actions -->
      <div class="flex gap-1.5">
        <button
          class="inline-flex cursor-pointer items-center gap-1 rounded-md border border-success/30 bg-success/15 px-2.5 py-1 text-xs font-medium text-success transition-all duration-150 hover:bg-success/25 disabled:cursor-not-allowed disabled:opacity-50"
          onclick={onApplyAllClick}
          disabled={unappliedTweaks.length === 0 || $isLoading || isBatchProcessing}
          title="Apply all unapplied tweaks in this category"
        >
          {#if isBatchProcessing}
            <Icon icon="mdi:loading" width="14" class="animate-spin" />
          {:else}
            <Icon icon="mdi:check-all" width="14" />
          {/if}
          Apply All
        </button>
        <button
          class="inline-flex cursor-pointer items-center gap-1 rounded-md border border-error/30 bg-error/15 px-2.5 py-1 text-xs font-medium text-error transition-all duration-150 hover:bg-error/25 disabled:cursor-not-allowed disabled:opacity-50"
          onclick={onRevertAllClick}
          disabled={appliedTweaks.length === 0 || $isLoading || isBatchProcessing}
          title="Revert all applied tweaks in this category"
        >
          {#if isBatchProcessing}
            <Icon icon="mdi:loading" width="14" class="animate-spin" />
          {:else}
            <Icon icon="mdi:undo-variant" width="14" />
          {/if}
          Revert All
        </button>
      </div>

      <!-- Count badge -->
      <span
        class="rounded-full bg-[hsl(var(--muted))] px-2.5 py-1 text-xs font-medium text-foreground-muted"
      >
        {appliedCount}/{tweaks.length} applied
      </span>

      <!-- Expand button -->
      <button
        class="mr-2 flex cursor-pointer items-center justify-center rounded border-0 bg-transparent p-2 text-foreground-muted transition-all duration-150 hover:bg-[hsl(var(--muted))] hover:text-foreground"
        onclick={() => (isExpanded = !isExpanded)}
      >
        <Icon
          icon={isExpanded ? "mdi:chevron-up" : "mdi:chevron-down"}
          width="20"
          class="transition-transform duration-200"
        />
      </button>
    </div>
  </div>

  <!-- Tweaks grid -->
  {#if isExpanded}
    <div class="mt-2 grid gap-2 pl-4 md:grid-cols-2 xl:grid-cols-3">
      {#each tweaks as tweak (tweak.definition.id)}
        <TweakCard {tweak} />
      {/each}
    </div>
  {/if}
</section>

<ConfirmDialog
  open={showApplyAllDialog}
  title="Apply All {category.name} Tweaks"
  message="This will apply {unappliedTweaks.length} tweak{unappliedTweaks.length === 1
    ? ''
    : 's'} in the {category.name} category. Some tweaks may require administrator privileges or a system restart."
  confirmText="Apply All"
  variant="warning"
  onconfirm={handleApplyAll}
  oncancel={() => (showApplyAllDialog = false)}
/>

<ConfirmDialog
  open={showRevertAllDialog}
  title="Revert All {category.name} Tweaks"
  message="This will revert {appliedTweaks.length} applied tweak{appliedTweaks.length === 1
    ? ''
    : 's'} in the {category.name} category to their default values."
  confirmText="Revert All"
  variant="danger"
  onconfirm={handleRevertAll}
  oncancel={() => (showRevertAllDialog = false)}
/>
