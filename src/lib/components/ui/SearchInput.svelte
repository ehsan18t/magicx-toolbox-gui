<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import IconButton from "./IconButton.svelte";
  import { field } from "./variants";

  interface Props {
    value: string;
    placeholder: string;
    class?: string;
    /** Accessible name; defaults to the placeholder. */
    label?: string;
    inputRef?: HTMLInputElement | null;
    onchange?: (value: string) => void;
    /** After the clear button, e.g. a scope toggle. */
    trailing?: Snippet | undefined;
    /** Backspace in an empty box, e.g. to drop that scope. */
    onbackspace?: (() => void) | undefined;
  }

  let {
    value,
    placeholder,
    class: className,
    label,
    inputRef = $bindable(null),
    onchange,
    trailing,
    onbackspace,
  }: Props = $props();

  // `for` pins the label to the input: unpinned, a click is forwarded to a leading button instead.
  const inputId = $props.id();

  function handleClear() {
    onchange?.("");
    inputRef?.focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && value) {
      e.preventDefault();
      handleClear();
    } else if (e.key === "Backspace" && !value && onbackspace) {
      e.preventDefault();
      onbackspace();
    }
  }
</script>

<label for={inputId} class={field({ focus: "within", class: className })}>
  <Icon icon="mdi:magnify" size="md" class="shrink-0 text-foreground-muted" />
  <input
    bind:this={inputRef}
    id={inputId}
    type="text"
    {placeholder}
    {value}
    aria-label={label ?? placeholder}
    oninput={(e) => onchange?.(e.currentTarget.value)}
    onkeydown={handleKeydown}
    class="min-w-0 flex-1 border-0 bg-transparent text-ui text-foreground outline-none placeholder:text-foreground-subtle"
  />
  {#if value}
    <IconButton icon="mdi:close" size="xs" label="Clear search" class="animate-pop-in" onclick={handleClear} />
  {/if}
  {@render trailing?.()}
</label>
