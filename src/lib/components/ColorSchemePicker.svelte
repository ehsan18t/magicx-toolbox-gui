<script lang="ts">
  import { COLOR_SCHEMES, colorSchemeStore, type ColorSchemeId } from "$lib/stores/colorScheme.svelte";
  import { cn } from "@/utils";

  interface Props {
    /** Size of color circles */
    size?: "sm" | "md";
  }

  const { size = "sm" }: Props = $props();

  const currentScheme = $derived(colorSchemeStore.current);

  const sizeClasses = {
    sm: "w-4 h-4",
    md: "w-5 h-5",
  };

  function handleSchemeChange(schemeId: ColorSchemeId) {
    colorSchemeStore.setScheme(schemeId);
  }
</script>

<div class="flex items-center gap-1.5">
  {#each COLOR_SCHEMES as scheme (scheme.id)}
    <button
      type="button"
      onclick={() => handleSchemeChange(scheme.id)}
      class={cn(
        "flex cursor-pointer items-center justify-center rounded-full transition-all duration-200",
        "hover:scale-110 hover:ring-2 hover:ring-white/30",
        "focus:ring-2 focus:ring-white/50 focus:outline-none",
        sizeClasses[size],
        currentScheme === scheme.id && "scale-110 ring-2 ring-white/60",
      )}
      style="background-color: {scheme.color}"
      title={scheme.name}
      aria-label="Set {scheme.name} color scheme"
      aria-pressed={currentScheme === scheme.id}
    >
      {#if currentScheme === scheme.id}
        <svg class="h-2.5 w-2.5 text-white drop-shadow-sm" viewBox="0 0 20 20" fill="currentColor">
          <path
            fill-rule="evenodd"
            d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
            clip-rule="evenodd"
          />
        </svg>
      {/if}
    </button>
  {/each}
</div>
