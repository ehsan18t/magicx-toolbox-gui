<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button } from "$lib/components/ui";
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
  disabled={tweakActionsStore.isBusy || tweaks.length === 0}
  onclick={() => tweakActionsStore.restoreAllWithConfirm(title, tweaks)}
>
  <Icon icon="mdi:history" width="16" />
  Restore all
  <span class="text-xs text-foreground-subtle tabular-nums">{tweaks.length}</span>
</Button>
