<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";

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

<button
  type="button"
  class="inline-flex cursor-pointer items-center gap-1 rounded-full border border-warning/35 bg-warning/10 py-0.5 pr-1 pl-2 font-medium text-warning enabled:hover:bg-warning/20 disabled:cursor-default disabled:opacity-70"
  aria-expanded={open}
  aria-controls={open ? controls : undefined}
  disabled={lockedReason !== null}
  use:tooltip={lockedReason ?? (open ? "Hide warning" : "Show warning")}
  onclick={ontoggle}
>
  <Icon icon="mdi:alert" size="2xs" class="shrink-0" />
  Warning
  <Icon
    icon="mdi:chevron-down"
    size="xs"
    class="shrink-0 transition-transform duration-normal {open ? 'rotate-180' : ''}"
  />
</button>
