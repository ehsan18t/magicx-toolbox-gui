<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { DIMMED } from "./variants";

  // Decorative: the selectable row around it carries the role and state.
  interface Props {
    checked: boolean;
    disabled?: boolean;
    class?: string;
  }

  let { checked, disabled = false, class: className }: Props = $props();
</script>

<span
  aria-hidden="true"
  class={cn(
    "flex h-5 w-5 shrink-0 items-center justify-center rounded border-0 ring-2 transition ring-inset",
    disabled && DIMMED,
    checked ? "bg-accent ring-accent" : "bg-transparent ring-border",
    className,
  )}
>
  {#if checked}
    <Icon icon="mdi:check" size="3xs" class="pointer-events-none animate-pop-in text-accent-foreground" />
  {/if}
</span>
