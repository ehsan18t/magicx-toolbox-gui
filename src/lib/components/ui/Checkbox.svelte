<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    checked: boolean;
    disabled?: boolean;
    indeterminate?: boolean;
    class?: string;
    "aria-label"?: string;
    onchange?: (checked: boolean) => void;
  }

  const {
    checked,
    disabled = false,
    indeterminate = false,
    class: className = "",
    "aria-label": ariaLabel = "Toggle option",
    onchange,
  }: Props = $props();

  function handleClick(e: MouseEvent) {
    e.stopPropagation();
    if (!disabled) {
      onchange?.(!checked);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.key === "Enter" || e.key === " ") && !disabled) {
      e.preventDefault();
      e.stopPropagation();
      onchange?.(!checked);
    }
  }
</script>

<button
  type="button"
  role="checkbox"
  aria-checked={indeterminate ? "mixed" : checked}
  aria-label={ariaLabel}
  {disabled}
  class="checkbox flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border-0 ring-2 transition-all duration-150 ring-inset
    focus-visible:ring-accent focus-visible:ring-offset-1
    {disabled ? 'cursor-not-allowed opacity-50' : 'hover:ring-accent'}
    {checked || indeterminate ? 'bg-accent ring-accent' : 'bg-transparent ring-border'}
    {className}"
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  {#if checked}
    <Icon icon="mdi:check" width={12} class="text-white" />
  {:else if indeterminate}
    <Icon icon="tabler:minus" width={12} class="text-white" />
  {/if}
</button>
