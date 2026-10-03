<script lang="ts">
  import { Button, type ButtonVariants } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { isHighRisk, restoreState } from "$lib/utils/tweakPresentation";
  import RowAction from "./RowAction.svelte";

  interface Props {
    tweak: TweakWithStatus;
    size?: ButtonVariants["size"];
    /** A row's ghost action, which drops its label in a narrow row. */
    row?: boolean;
  }

  let { tweak, size, row = false }: Props = $props();

  const def = $derived(tweak.definition);
  const restore = $derived(restoreState(def, tweak.status, tweakActionsStore.isRunning(def.id)));
  const name = $derived(`${restore.label} ${def.name}`);

  const onclick = () => tweakActionsStore.restoreWithConfirm(def, isHighRisk(def.riskLevel));
</script>

{#if row}
  <RowAction
    icon="mdi:history"
    label={restore.label}
    aria-label={name}
    tooltip={restore.tip}
    disabled={restore.disabled}
    collapses
    {onclick}
  />
{:else}
  <Button {size} icon="mdi:history" disabled={restore.disabled} tooltip={restore.tip} aria-label={name} {onclick}>
    {restore.label}
  </Button>
{/if}
