<script lang="ts">
  import Icon from "@iconify/svelte";

  const {
    title,
    icon,
    variant = "default",
    onClick,
  } = $props<{
    title: string;
    icon: string;
    variant?: "default" | "theme" | "danger";
    onClick: () => void;
  }>();
</script>

<button class="control-btn {variant}" {title} onclick={onClick}>
  <div class="icon-wrapper">
    <Icon {icon} width="16" height="16" />
  </div>
  <div class="ripple"></div>
</button>

<style>
  .control-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 4px;
    background-color: transparent;
    cursor: pointer;
    overflow: hidden;
    transition: background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
    -webkit-tap-highlight-color: transparent;
    outline: none;
  }

  /* Icon wrapper */
  .icon-wrapper {
    position: relative;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .icon-wrapper :global(svg) {
    color: hsl(var(--foreground-muted));
    transition: color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* Material ripple effect */
  .ripple {
    position: absolute;
    border-radius: 50%;
    background-color: currentColor;
    opacity: 0;
    pointer-events: none;
    transform: scale(0);
    animation: none;
  }

  /* Default variant - subtle gray */
  .control-btn.default:hover {
    background-color: hsl(var(--foreground) / 0.08);
  }

  .control-btn.default:hover .icon-wrapper :global(svg) {
    color: hsl(var(--foreground));
  }

  .control-btn.default:active {
    background-color: hsl(var(--foreground) / 0.12);
  }

  /* Theme variant - accent color */
  .control-btn.theme {
    background-color: transparent;
  }

  .control-btn.theme:hover {
    background-color: hsl(var(--accent) / 0.12);
  }

  .control-btn.theme:hover .icon-wrapper :global(svg) {
    color: hsl(var(--accent));
  }

  .control-btn.theme:active {
    background-color: hsl(var(--accent) / 0.18);
  }

  /* Danger variant - error color */
  .control-btn.danger:hover {
    background-color: hsl(var(--error) / 0.12);
  }

  .control-btn.danger:hover .icon-wrapper :global(svg) {
    color: hsl(var(--error));
  }

  .control-btn.danger:active {
    background-color: hsl(var(--error) / 0.18);
  }

  /* Focus states - Material Design focus rings */
  .control-btn:focus-visible {
    background-color: hsl(var(--foreground) / 0.12);
    outline: none;
  }

  .control-btn.theme:focus-visible {
    background-color: hsl(var(--accent) / 0.18);
  }

  .control-btn.danger:focus-visible {
    background-color: hsl(var(--error) / 0.18);
  }

  /* Pressed state */
  .control-btn:active .icon-wrapper {
    transform: scale(0.9);
  }

  /* Disabled state */
  .control-btn:disabled {
    opacity: 0.38;
    cursor: default;
    pointer-events: none;
  }

  .control-btn:disabled .icon-wrapper :global(svg) {
    color: hsl(var(--foreground) / 0.38);
  }
</style>
