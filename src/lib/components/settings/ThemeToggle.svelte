<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { duration } from "$lib/utils/motion";

  let isAnimating = $state(false);

  const toggleTheme = () => {
    if (isAnimating) return;
    isAnimating = true;
    themeStore.toggle();
    setTimeout(() => (isAnimating = false), duration("slow"));
  };
</script>

<button
  type="button"
  use:tooltip={`Switch to ${themeStore.current === "dark" ? "light" : "dark"} mode`}
  aria-label={`Switch to ${themeStore.current === "dark" ? "light" : "dark"} mode`}
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
      background-color var(--transition-duration-fast) var(--ease-out),
      transform var(--transition-duration-fast) var(--ease-out);
  }

  .theme-toggle:hover {
    background-color: hsl(var(--accent) / 0.12);
  }

  .theme-toggle:active,
  .theme-toggle.is-animating {
    transform: scale(calc(1 - 2 * var(--motion-scale-delta)));
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
      opacity var(--transition-duration-normal) var(--ease-out),
      transform var(--transition-duration-slow) var(--ease-overshoot),
      color var(--transition-duration-fast) var(--ease-out);
  }

  .icon.active {
    opacity: 1;
    transform: rotate(0deg) scale(1);
  }

  .theme-toggle:hover .icon.active {
    color: hsl(var(--accent));
  }
</style>
