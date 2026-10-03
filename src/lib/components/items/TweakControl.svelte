<script lang="ts" module>
  import type { TweakStatus } from "$lib/types";
  import {
    CHECKING,
    labelsOf,
    SYSTEM_DEFAULT_LABEL,
    unavailableReason,
    usesDropdown,
  } from "$lib/utils/tweakPresentation";

  // ADR-0003: System Default joins a control only beside a lone option, where choosing it restores
  // the snapshot; elsewhere the state line names it and Restore is the way back.
  const SYSTEM_DEFAULT = "__system_default__";

  const PLACEHOLDER: Partial<Record<TweakStatus["state"], string>> = {
    system_default: SYSTEM_DEFAULT_LABEL,
    loading: CHECKING.label,
    unavailable: "Unavailable",
  };
</script>

<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { SegmentedSwitch, type SegmentOption, Select } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { cn } from "$lib/utils/cn";

  interface Props {
    tweak: TweakWithStatus;
    class?: string;
  }

  let { tweak, class: className }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const isLoading = $derived(tweakActionsStore.isRunning(def.id));
  const hasSnapshot = $derived(status.hasHistory);
  const pendingChange = $derived(pendingChangesStore.change(def.id));
  const hasPending = $derived(pendingChange !== undefined);
  const activeOption = $derived(status.activeOption);
  const optionLabels = $derived(labelsOf(def));
  const dropdown = $derived(usesDropdown(def));
  const selected = $derived(
    pendingChange?.optionLabel ?? activeOption ?? (status.state === "system_default" ? SYSTEM_DEFAULT : null),
  );

  const disabledReason = $derived.by(() => {
    if (status.state === "unavailable") return unavailableReason(status);
    if (def.availability.state !== "available") return def.availability.reason;
    return null;
  });
  const disabled = $derived(isLoading || disabledReason !== null);

  const onlyOption = $derived(optionLabels.length === 1 ? optionLabels[0] : null);
  const defaultBlocked = $derived(onlyOption !== null && activeOption === onlyOption && !hasPending && !hasSnapshot);
  const options = $derived.by(() => {
    const list: SegmentOption<string>[] = optionLabels.map((label) => {
      const unavailable = status.unavailableOptions.some((u) => u.label === label);
      return { value: label, label: unavailable ? `${label} (unavailable)` : label, disabled: unavailable };
    });
    if (onlyOption !== null) {
      list.unshift({
        value: SYSTEM_DEFAULT,
        label: SYSTEM_DEFAULT_LABEL,
        disabled: defaultBlocked,
        tooltip: defaultBlocked ? "Already set before a snapshot was saved, so there is nothing to restore" : undefined,
        confirms: true,
      });
    }
    return list;
  });

  function choose(target: string) {
    if (target === SYSTEM_DEFAULT) {
      if (hasPending) pendingChangesStore.remove(def.id);
      // Every other segment only stages; this one changes the system at once, so it always asks.
      else if (hasSnapshot) void tweakActionsStore.restoreWithConfirm(def, true);
    } else if (target === activeOption) pendingChangesStore.remove(def.id);
    else pendingChangesStore.stage(def.id, target);
  }
</script>

<div class={cn("min-w-0", dropdown && "w-fit min-w-44", className)} use:tooltip={disabledReason}>
  {#if dropdown}
    <Select
      value={selected}
      {options}
      placeholder={PLACEHOLDER[status.state] ?? "Unknown"}
      pending={hasPending}
      loading={isLoading}
      {disabled}
      label={def.name}
      onchange={choose}
    />
  {:else}
    <SegmentedSwitch
      value={selected}
      {options}
      pending={hasPending}
      loading={isLoading}
      {disabled}
      label={def.name}
      onchange={choose}
    />
  {/if}
</div>
