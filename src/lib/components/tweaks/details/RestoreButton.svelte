<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { Button, ICON_SIZE } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { isHighRisk, restoreState } from "$lib/utils/tweakPresentation";
  import { fromAction } from "svelte/attachments";

  let { tweak }: { tweak: TweakWithStatus } = $props();

  const def = $derived(tweak.definition);
  const restore = $derived(restoreState(def, tweak.status, tweakActionsStore.isRunning(def.id)));
</script>

<Button
  disabled={restore.disabled}
  {@attach fromAction(tooltip, () => restore.tip)}
  onclick={() => tweakActionsStore.restoreWithConfirm(def, isHighRisk(def.riskLevel))}
>
  <Icon icon="mdi:history" width={ICON_SIZE.md} />
  {tweak.status.attention?.reason === "restore_failed" ? "Retry restore" : "Restore"}
</Button>
