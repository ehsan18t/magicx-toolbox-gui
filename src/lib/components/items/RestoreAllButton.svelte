<script lang="ts">
  import { Button, Count } from "$lib/components/ui";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import type { TweakWithStatus } from "$lib/types";

  interface Props {
    /** The confirmation's title, e.g. "Restore favorites?". */
    title: string;
    /** The restorable tweaks; none disables the button. */
    tweaks: TweakWithStatus[];
    class?: string;
  }

  let { title, tweaks, class: className }: Props = $props();
</script>

<Button
  class={className}
  icon="mdi:history"
  disabled={tweakActionsStore.isBusy || tweaks.length === 0}
  onclick={() => tweakActionsStore.restoreAllWithConfirm(title, tweaks)}
>
  Restore all
  <Count value={tweaks.length} />
</Button>
