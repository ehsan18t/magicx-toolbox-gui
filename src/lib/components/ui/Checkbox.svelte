<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";

  // Decorative: the selectable row around it carries the role and state.
  interface Props {
    checked: boolean;
    disabled?: boolean;
  }

  const { checked, disabled = false }: Props = $props();
</script>

<span
  aria-hidden="true"
  class={cn(
    "flex h-5 w-5 shrink-0 items-center justify-center rounded border-0 ring-2 transition ring-inset",
    disabled && "opacity-50",
    checked ? "bg-accent ring-accent" : "bg-transparent ring-border",
  )}
>
  {#if checked}
    <Icon icon="mdi:check" width={12} class="pointer-events-none animate-pop-in text-white" />
  {/if}
</span>
