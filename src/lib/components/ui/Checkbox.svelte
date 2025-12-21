<script lang="ts">
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
    "aria-label": ariaLabel,
    onchange,
  }: Props = $props();

  function handleClick(e: MouseEvent) {
    // Stop propagation to prevent parent buttons from also triggering
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
  class="checkbox flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border-2 transition-all duration-150
    {disabled ? 'cursor-not-allowed opacity-50' : 'hover:border-accent'}
    {checked || indeterminate ? 'border-accent bg-accent' : 'border-border bg-transparent'}
    {className}"
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  {#if checked}
    <svg class="h-3 w-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
      <polyline points="20 6 9 17 4 12"></polyline>
    </svg>
  {:else if indeterminate}
    <svg class="h-3 w-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
      <line x1="5" y1="12" x2="19" y2="12"></line>
    </svg>
  {/if}
</button>
