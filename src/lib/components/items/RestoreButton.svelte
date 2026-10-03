<script lang="ts">
  import { Button, type ButtonVariants } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { isHighRisk, restoreState } from "$lib/utils/tweakPresentation";

  interface Props {
    tweak: TweakWithStatus;
    size?: ButtonVariants["size"];
  }

  let { tweak, size }: Props = $props();

  const def = $derived(tweak.definition);
  const restore = $derived(restoreState(def, tweak.status, tweakActionsStore.isRunning(def.id)));
</script>

<Button
  {size}
  icon="mdi:history"
  disabled={restore.disabled}
  tooltip={restore.tip}
  onclick={() => tweakActionsStore.restoreWithConfirm(def, isHighRisk(def.riskLevel))}
>
  {restore.label}
</Button>
