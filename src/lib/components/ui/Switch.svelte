<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { ICON_SIZE } from "./icon";
  import { DISABLED } from "./variants";

  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    loading?: boolean;
    class?: string;
    onchange?: (checked: boolean) => void;
  }

  let { checked, label, disabled = false, loading = false, class: className, onchange }: Props = $props();

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
  aria-busy={loading}
  {disabled}
  class={cn("shrink-0 cursor-pointer border-0 bg-transparent p-0", DISABLED, className)}
  onclick={toggle}
  onkeydown={handleKeydown}
>
  <span
    class="flex h-6 w-11 items-center rounded-full p-0.5 transition-[background-color,filter] duration-normal hover:brightness-95
      {checked ? 'bg-accent' : 'bg-muted'}"
  >
    <span
      class="flex h-5 w-5 items-center justify-center rounded-full shadow-md transition-transform duration-normal
        {checked ? 'translate-x-5 bg-accent-foreground' : 'translate-x-0 bg-thumb'}
        {loading ? 'text-foreground-muted' : 'text-accent'}"
    >
      {#if loading}
        <Icon icon="mdi:loading" width={ICON_SIZE.sm} class="animate-spin" />
      {:else if checked}
        <Icon icon="mdi:check" width={ICON_SIZE.sm} class="animate-pop-in" />
      {/if}
    </span>
  </span>
</button>
