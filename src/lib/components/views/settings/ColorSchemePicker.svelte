<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { radioKeyIndex } from "$lib/components/ui";
  import { COLOR_SCHEMES, colorSchemeStore } from "$lib/stores/colorScheme.svelte";

  function handleKeydown(e: KeyboardEvent & { currentTarget: HTMLElement }) {
    const from = COLOR_SCHEMES.findIndex((s) => s.id === colorSchemeStore.current);
    const next = radioKeyIndex(COLOR_SCHEMES, from, e.key);
    if (next === null) return;
    e.preventDefault();
    colorSchemeStore.set(COLOR_SCHEMES[next].id);
    e.currentTarget.querySelectorAll<HTMLElement>("[role='radio']")[next]?.focus();
  }
</script>

<!-- Svelte requires a tabindex on a radiogroup that takes keys; -1 keeps the group itself out of the tab order. -->
<div
  role="radiogroup"
  aria-label="Accent color"
  tabindex="-1"
  class="flex items-center gap-1.5"
  onkeydown={handleKeydown}
>
  {#each COLOR_SCHEMES as scheme (scheme.id)}
    {@const selected = colorSchemeStore.current === scheme.id}
    <!-- The ::before widens the hit area to 24px without changing the swatch. -->
    <button
      type="button"
      role="radio"
      aria-checked={selected}
      tabindex={selected ? 0 : -1}
      onclick={() => colorSchemeStore.set(scheme.id)}
      class={[
        "relative flex h-5 w-5 cursor-pointer items-center justify-center rounded-full transition duration-normal before:absolute before:-inset-0.5",
        "hover:scale-110 hover:ring-2 hover:ring-border-hover",
        selected && "scale-110 ring-2 ring-foreground-muted",
      ]}
      style:background-color="var(--swatch-{scheme.id})"
      use:tooltip={scheme.name}
      aria-label={scheme.name}
    >
      {#if selected}
        <Icon icon="mdi:check" size="3xs" class="animate-pop-in text-accent-foreground" />
      {/if}
    </button>
  {/each}
</div>
