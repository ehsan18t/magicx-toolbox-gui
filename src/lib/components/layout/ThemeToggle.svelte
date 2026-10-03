<script lang="ts">
  import { tooltip } from "$lib/attachments/tooltip.svelte";
  import { Icon } from "$lib/components/shared";
  import { iconButton } from "$lib/components/ui";
  import type { IconName } from "$lib/design";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { duration } from "$lib/utils/motion";
  import { onDestroy } from "svelte";

  let isAnimating = $state(false);
  let settleTimer: ReturnType<typeof setTimeout> | undefined;
  const label = $derived(`Switch to ${themeStore.current === "dark" ? "light" : "dark"} mode`);

  onDestroy(() => clearTimeout(settleTimer));

  function toggleTheme() {
    if (isAnimating) return;
    isAnimating = true;
    themeStore.toggle();
    settleTimer = setTimeout(() => (isAnimating = false), duration("slow"));
  }
</script>

{#snippet icon(name: IconName, active: boolean)}
  <span
    class={[
      "theme-icon absolute inset-0 flex items-center justify-center",
      active ? "group-hover:text-accent" : "scale-50 -rotate-90 opacity-0",
    ]}
  >
    <Icon icon={name} size="md" />
  </span>
{/snippet}

<!-- `theme-toggle` keeps its own motion while app.css freezes every other transition during a theme swap. -->
<button
  type="button"
  {@attach tooltip(() => label)}
  aria-label={label}
  onclick={toggleTheme}
  data-pressed={isAnimating || undefined}
  class={iconButton({
    class:
      "theme-toggle group relative transition-[background-color,scale] active:scale-pressed data-pressed:scale-pressed",
  })}
>
  <span class="relative h-4 w-4">
    {@render icon("tabler:moon", themeStore.current === "light")}
    {@render icon("tabler:sun", themeStore.current === "dark")}
  </span>
</button>

<style>
  /* One timing per property (the swap overshoots, the fade and hover colour do not): utilities set one. */
  .theme-icon {
    transition:
      opacity var(--transition-duration-normal) var(--ease-out),
      rotate var(--transition-duration-slow) var(--ease-overshoot),
      scale var(--transition-duration-slow) var(--ease-overshoot),
      color var(--transition-duration-fast) var(--ease-out);
  }
</style>
