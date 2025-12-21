<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import Icon from "../Icon.svelte";
  import { statusBadge, type StatusBadgeVariants } from "./variants";

  interface Props {
    /** Badge variant/color */
    variant: StatusBadgeVariants["variant"];
    /** Icon name (mdi icon) */
    icon?: string;
    /** Label text */
    label: string;
    /** Tooltip text */
    tooltip?: string;
    /** Additional classes */
    class?: string;
  }

  const { variant, icon, label, tooltip: tooltipText, class: className = "" }: Props = $props();
</script>

<div class={statusBadge({ variant, class: className })} use:tooltip={tooltipText}>
  {#if icon}
    <Icon {icon} width="12" />
  {/if}
  <span>{label}</span>
</div>
