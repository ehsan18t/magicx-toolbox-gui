<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    value: string;
    placeholder?: string;
    class?: string;
    /** Accessible name; defaults to the placeholder. */
    label?: string;
    inputRef?: HTMLInputElement | null;
    onchange?: (value: string) => void;
    onclear?: () => void;
  }

  let {
    value = "",
    placeholder = "Search...",
    class: className = "",
    label,
    inputRef = $bindable(null),
    onchange,
    onclear,
  }: Props = $props();

  function handleClear() {
    onchange?.("");
    onclear?.();
    inputRef?.focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && value) {
      e.preventDefault();
      handleClear();
    }
  }
</script>

<label
  class="flex h-8 min-w-0 items-center gap-2 rounded-md border border-border bg-secondary px-2.5 focus-within:border-accent {className}"
>
  <Icon icon="mdi:magnify" width="16" class="shrink-0 text-foreground-muted" />
  <input
    bind:this={inputRef}
    type="text"
    {placeholder}
    {value}
    aria-label={label ?? placeholder}
    oninput={(e) => onchange?.(e.currentTarget.value)}
    onkeydown={handleKeydown}
    class="min-w-0 flex-1 border-0 bg-transparent text-[13px] text-foreground outline-none placeholder:text-foreground-subtle"
  />
  {#if value}
    <button
      type="button"
      onclick={handleClear}
      class="flex shrink-0 cursor-pointer rounded p-0.5 text-foreground-muted hover:bg-muted hover:text-foreground"
      aria-label="Clear search"
    >
      <Icon icon="mdi:close" width="14" />
    </button>
  {/if}
</label>
