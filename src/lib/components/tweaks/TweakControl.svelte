<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { SegmentedSwitch, Select } from "$lib/components/ui";
  import { tweakOps } from "$lib/stores/tweakOps.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { restoreWithConfirm } from "$lib/utils/tweakActions";
  import { labelsOf } from "$lib/utils/tweakPresentation";

  interface Props {
    tweak: TweakWithStatus;
    class?: string;
  }

  let { tweak, class: className = "" }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const isLoading = $derived(tweakOps.isRunning(def.id));
  const hasSnapshot = $derived(status.hasSnapshot);
  const pendingChange = $derived(pendingChangesStore.get(def.id));
  const hasPending = $derived(pendingChange !== undefined);
  const activeOption = $derived(status.activeOption);
  const optionLabels = $derived(labelsOf(def));
  const selectValue = $derived(pendingChange?.optionLabel ?? activeOption);

  const disabledReason = $derived.by(() => {
    if (status.state === "unavailable") return status.unavailableReason ?? "Not available on this system";
    if (def.availability.state !== "available") return def.availability.reason;
    return null;
  });
  const disabled = $derived(isLoading || disabledReason !== null);

  // ADR-0003: System Default joins a control only beside a lone option, where choosing it restores
  // the snapshot; elsewhere the state line names it and Restore is the way back.
  const SYSTEM_DEFAULT = "__system_default__";
  const onlyOption = $derived(optionLabels.length === 1 ? optionLabels[0] : null);
  const defaultBlocked = $derived(onlyOption !== null && activeOption === onlyOption && !hasPending && !hasSnapshot);
  const segments = $derived.by(() => {
    const options: { target: string; label: string; disabled: boolean; tip?: string }[] = optionLabels.map((label) => {
      const unavailable = status.unavailableOptions.some((u) => u.label === label);
      return { target: label, label: unavailable ? `${label} (unavailable)` : label, disabled: unavailable };
    });
    if (onlyOption !== null) {
      options.unshift({
        target: SYSTEM_DEFAULT,
        label: "System default",
        disabled: defaultBlocked,
        tip: defaultBlocked ? "Already set before a snapshot was saved, so there is nothing to restore" : undefined,
      });
    }
    return options.map((o, i) => ({ ...o, value: i }));
  });
  const segmentValue = $derived(
    segments.findIndex(
      (s) => s.target === (selectValue ?? (status.state === "system_default" ? SYSTEM_DEFAULT : null)),
    ),
  );
  const selectOptions = $derived(segments.map((s) => ({ ...s, value: s.target })));
  const selectPlaceholder = $derived(
    status.state === "system_default"
      ? "System default"
      : status.state === "loading"
        ? "Checking…"
        : status.state === "unavailable"
          ? "Unavailable"
          : "Unknown",
  );

  function selectTarget(target: string) {
    if (target === SYSTEM_DEFAULT) {
      if (hasPending) pendingChangesStore.remove(def.id);
      // Every other segment only stages; this one changes the system at once, so it always asks.
      else if (hasSnapshot) void restoreWithConfirm(def, true);
    } else if (target === activeOption) pendingChangesStore.remove(def.id);
    else pendingChangesStore.set(def.id, { tweakId: def.id, optionLabel: target });
  }
</script>

<div class="min-w-0 {optionLabels.length > 2 ? 'w-fit min-w-44' : ''} {className}" use:tooltip={disabledReason}>
  {#if optionLabels.length <= 2}
    <SegmentedSwitch
      value={segmentValue}
      options={segments}
      pending={hasPending}
      loading={isLoading}
      {disabled}
      label={def.name}
      onchange={(i) => selectTarget(segments[i].target)}
    />
  {:else}
    <Select
      value={selectValue}
      options={selectOptions}
      placeholder={selectPlaceholder}
      pending={hasPending}
      loading={isLoading}
      {disabled}
      label={def.name}
      onchange={(v) => selectTarget(String(v))}
    />
  {/if}
</div>
