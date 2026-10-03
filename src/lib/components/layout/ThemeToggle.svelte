<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon, type IconName } from "$lib/components/shared";
  import { ICON_SIZE } from "$lib/components/ui";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { duration } from "$lib/utils/motion";

  let isAnimating = $state(false);
  const label = $derived(`Switch to ${themeStore.current === "dark" ? "light" : "dark"} mode`);

  function toggleTheme() {
    if (isAnimating) return;
    isAnimating = true;
    themeStore.toggle();
    setTimeout(() => (isAnimating = false), duration("slow"));
  }
</script>

{#snippet icon(name: IconName, active: boolean)}
  <span
    class={[
      "theme-icon absolute inset-0 flex items-center justify-center text-foreground-muted",
      active ? "group-hover:text-accent" : "scale-50 -rotate-90 opacity-0",
    ]}
  >
    <Icon icon={name} width={ICON_SIZE.md} />
  </span>
{/snippet}

<!-- `theme-toggle` keeps its own motion while app.css freezes every other transition during a theme swap. -->
<button
  type="button"
  use:tooltip={label}
  aria-label={label}
  onclick={toggleTheme}
  data-pressed={isAnimating || undefined}
  class="theme-toggle group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-md border-0 bg-transparent transition-[background-color,scale] hover:bg-accent/12 active:scale-[calc(1-2*var(--motion-scale-delta))] data-pressed:scale-[calc(1-2*var(--motion-scale-delta))]"
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
