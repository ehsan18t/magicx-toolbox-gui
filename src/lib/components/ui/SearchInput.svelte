<script lang="ts">
  import Icon from "../Icon.svelte";

  interface Props {
    value: string;
    placeholder?: string;
    class?: string;
    onchange?: (value: string) => void;
    onclear?: () => void;
  }

  const { value = "", placeholder = "Search...", class: className = "", onchange, onclear }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onchange?.(target.value);
  }

  function handleClear() {
    onchange?.("");
    onclear?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && value) {
      handleClear();
    }
  }
</script>

<div class="relative {className}">
  <Icon
    icon="mdi:magnify"
    width="18"
    class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-foreground-muted"
  />
  <input
    type="text"
    {placeholder}
    {value}
    oninput={handleInput}
    onkeydown={handleKeydown}
    class="w-full rounded-lg border border-border bg-surface py-2 pr-9 pl-10 text-sm text-foreground placeholder:text-foreground-muted focus:border-accent focus:ring-2 focus:ring-accent/20 focus:outline-none"
  />
  {#if value}
    <button
      type="button"
      onclick={handleClear}
      class="hover:bg-muted absolute top-1/2 right-2 flex h-6 w-6 -translate-y-1/2 cursor-pointer items-center justify-center rounded border-0 bg-transparent text-foreground-muted transition-colors hover:text-foreground"
      aria-label="Clear search"
    >
      <Icon icon="mdi:close" width="16" />
    </button>
  {/if}
</div>
