<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    loading?: boolean;
    onchange?: (checked: boolean) => void;
  }

  const { checked, label, disabled = false, loading = false, onchange }: Props = $props();

  function toggle() {
    if (!disabled && !loading) onchange?.(!checked);
  }

  // Toggles on keydown: the native click fires on Space keyup, a beat later.
  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    toggle();
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  class="shrink-0 cursor-pointer border-0 bg-transparent p-0 disabled:cursor-not-allowed disabled:opacity-70"
  onclick={toggle}
  onkeydown={handleKeydown}
>
  <span
    class="flex h-6 w-11 items-center rounded-full p-0.5 transition-[background-color,filter] duration-normal hover:brightness-95
      {checked ? 'bg-accent' : 'bg-muted'}"
  >
    <span
      class="flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-md transition-transform duration-normal
        {checked ? 'translate-x-5' : 'translate-x-0'}
        {loading ? 'text-foreground-muted' : 'text-accent'}"
    >
      {#if loading}
        <Icon icon="mdi:loading" width={14} class="animate-spin" />
      {:else if checked}
        <Icon icon="mdi:check" width={14} class="animate-pop-in" />
      {/if}
    </span>
  </span>
</button>
