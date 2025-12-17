<script lang="ts">
  import { themeStore } from "$lib/stores/theme.svelte";
  import { onDestroy } from "svelte";
  import Icon from "./Icon.svelte";

  let isAnimating = $state(false);
  let animationTimer: ReturnType<typeof setTimeout> | null = null;

  const toggleTheme = () => {
    if (isAnimating) return;
    isAnimating = true;
    themeStore.toggle();
    // Reset animation state after transition completes
    if (animationTimer) {
      clearTimeout(animationTimer);
    }
    animationTimer = setTimeout(() => {
      isAnimating = false;
      animationTimer = null;
    }, 300);
  };

  onDestroy(() => {
    if (animationTimer) {
      clearTimeout(animationTimer);
      animationTimer = null;
    }
  });
</script>

<button
  type="button"
  title={`Switch to ${themeStore.isDark ? "light" : "dark"} mode`}
  aria-label={`Switch to ${themeStore.isDark ? "light" : "dark"} mode`}
  onclick={toggleTheme}
  class="theme-toggle group"
  class:is-animating={isAnimating}
>
  <span class="icon-wrapper">
    <span class="icon" class:active={themeStore.current === "light"}>
      <Icon icon="tabler:moon" width="16" height="16" />
    </span>
    <span class="icon" class:active={themeStore.current === "dark"}>
      <Icon icon="tabler:sun" width="16" height="16" />
    </span>
  </span>
</button>

<style>
  .theme-toggle {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: none;
    border-radius: 0.375rem;
    background: transparent;
    cursor: pointer;
    transition:
      background-color 150ms ease,
      transform 150ms ease;
  }

  .theme-toggle:hover {
    background-color: hsl(var(--accent) / 0.12);
  }

  .theme-toggle:active,
  .theme-toggle.is-animating {
    transform: scale(0.9);
  }

  .icon-wrapper {
    position: relative;
    width: 16px;
    height: 16px;
  }

  .icon {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: hsl(var(--foreground-muted));
    opacity: 0;
    transform: rotate(-90deg) scale(0.5);
    transition:
      opacity 200ms ease,
      transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1),
      color 150ms ease;
  }

  .icon.active {
    opacity: 1;
    transform: rotate(0deg) scale(1);
  }

  .theme-toggle:hover .icon.active {
    color: hsl(var(--accent));
  }
</style>
