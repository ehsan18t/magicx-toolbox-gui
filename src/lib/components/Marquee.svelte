<script lang="ts">
  import { onMount } from "svelte";

  interface Props {
    children: import("svelte").Snippet;
    speed?: number; // pixels per second
    pauseOnHover?: boolean;
    fade?: boolean; // linear mask
  }

  let { children, speed = 30, pauseOnHover = true, fade = true }: Props = $props();

  let container: HTMLDivElement | undefined = $state();
  let content: HTMLDivElement | undefined = $state();
  let isOverflowing = $state(false);
  let duration = $state(0);

  const checkOverflow = () => {
    if (container && content) {
      const containerW = container.clientWidth;
      const contentW = content.scrollWidth;

      const newIsOverflowing = contentW > containerW;

      if (newIsOverflowing !== isOverflowing) {
        isOverflowing = newIsOverflowing;
      }

      if (contentW > 0) {
        duration = contentW / speed;
      }
    }
  };

  // Re-check when children or other props change, and on mount/resize
  $effect(() => {
    // This effect tracks dependencies automatically.
    // However, DOM measurements need to happen after render.
    // We'll rely mostly on ResizeObserver but call checkOverflow once here to catch prop updates.
    checkOverflow();
  });

  onMount(() => {
    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(checkOverflow);
    });

    if (container) resizeObserver.observe(container);
    if (content) resizeObserver.observe(content);

    // Initial check
    checkOverflow();

    return () => resizeObserver.disconnect();
  });
</script>

<div
  bind:this={container}
  class="relative isolate block w-full overflow-hidden"
  class:mask-fade={fade && isOverflowing}
>
  <div
    class="flex w-max"
    class:animate-marquee={isOverflowing}
    style:--duration="{duration}s"
    style:--play-state={pauseOnHover ? "paused" : "running"}
  >
    <div bind:this={content} class="flex items-center gap-2 pr-4">
      {@render children()}
    </div>
    {#if isOverflowing}
      <div class="flex items-center gap-2 pr-4" aria-hidden="true">
        {@render children()}
      </div>
    {/if}
  </div>
</div>

<style>
  .animate-marquee {
    animation: marquee var(--duration) linear infinite;
    animation-play-state: var(--play-state);
  }

  /* Only pause on hover if requested */
  .animate-marquee:hover {
    animation-play-state: paused;
  }

  @keyframes marquee {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-50%);
    }
  }

  .mask-fade {
    mask-image: linear-gradient(to right, transparent 0%, black 20px, black calc(100% - 20px), transparent 100%);
  }
</style>
