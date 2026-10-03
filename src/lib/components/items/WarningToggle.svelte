<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { HELD, ToggleChip } from "$lib/components/ui";

  interface Props {
    open: boolean;
    /** The WarningNotice's id. */
    controls: string;
    /** Holds the notice open and says why. */
    lockedReason?: string | null;
    ontoggle: () => void;
  }

  let { open, controls, lockedReason = null, ontoggle }: Props = $props();
</script>

<ToggleChip
  tone="warning"
  icon="mdi:alert"
  aria-expanded={open}
  aria-controls={open ? controls : undefined}
  disabled={lockedReason !== null}
  tooltip={lockedReason ?? (open ? "Hide warning" : "Show warning")}
  class={lockedReason !== null ? HELD : undefined}
  onclick={ontoggle}
>
  Warning
  <Icon
    icon="mdi:chevron-down"
    size="xs"
    class="shrink-0 transition-transform duration-normal {open ? 'rotate-180' : ''}"
  />
</ToggleChip>
