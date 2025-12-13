<script lang="ts">
  interface Props {
    checked: boolean;
    disabled?: boolean;
    loading?: boolean;
    pending?: boolean;
    class?: string;
    onchange?: (checked: boolean) => void;
  }

  const {
    checked,
    disabled = false,
    loading = false,
    pending = false,
    class: className = "",
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
        <svg class="animate-spin h-3.5 w-3.5" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
      {:else if checked}
        <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
        </svg>
      {/if}
    </span>
  </span>
</button>
