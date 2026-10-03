<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { COLOR_SCHEMES, colorSchemeStore } from "$lib/stores/colorScheme.svelte";
</script>

<div class="flex items-center gap-1.5">
  {#each COLOR_SCHEMES as scheme (scheme.id)}
    {@const selected = colorSchemeStore.current === scheme.id}
    <button
      type="button"
      onclick={() => colorSchemeStore.set(scheme.id)}
      class={[
        "flex h-5 w-5 cursor-pointer items-center justify-center rounded-full transition duration-normal",
        "hover:scale-110 hover:ring-2 hover:ring-border-hover",
        selected && "scale-110 ring-2 ring-foreground-muted",
      ]}
      style:background-color={scheme.color}
      use:tooltip={scheme.name}
      aria-label="Set {scheme.name} color scheme"
      aria-pressed={selected}
    >
      <!-- The selected scheme is the live accent, so accent-foreground reads on its swatch. -->
      {#if selected}
        <Icon icon="mdi:check" size="3xs" class="animate-pop-in text-accent-foreground" />
      {/if}
    </button>
  {/each}
</div>
