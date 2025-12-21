<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    checked: boolean;
    disabled?: boolean;
    loading?: boolean;
    pending?: boolean;
    class?: string;
    "aria-label"?: string;
    onchange?: (checked: boolean) => void;
  }

  const {
    checked,
    disabled = false,
    loading = false,
    pending = false,
    class: className = "",
    "aria-label": ariaLabel = "Toggle",
    onchange,
  }: Props = $props();

  function handleClick() {
    if (!disabled && !loading) {
      onchange?.(!checked);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.key === "Enter" || e.key === " ") && !disabled && !loading) {
      e.preventDefault();
      onchange?.(!checked);
    }
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={ariaLabel}
  {disabled}
  class="switch shrink-0 cursor-pointer border-0 bg-transparent p-0 disabled:cursor-not-allowed disabled:opacity-70 {className}"
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  <span
    class="switch-track flex h-6 w-11 items-center rounded-full p-0.5 transition-colors duration-200 hover:brightness-95
      {checked ? (pending ? 'bg-warning' : 'bg-accent') : 'bg-muted'}"
  >
    <span
      class="switch-thumb flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-md transition-transform duration-200
        {checked ? 'translate-x-5' : 'translate-x-0'}
        {loading ? 'text-foreground-muted' : 'text-accent'}"
    >
      {#if loading}
        <Icon icon="mdi:loading" width={14} class="animate-spin" />
      {:else if checked}
        <Icon icon="mdi:check" width={14} />
      {/if}
    </span>
  </span>
</button>
