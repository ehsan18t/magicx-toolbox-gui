<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import { button } from "./variants";

  /**
   * EmptyState - Consistent empty/no-results state display
   *
   * Used for no tweaks, no favorites, no snapshots, no search results, etc.
   */

  interface Props {
    /** Main icon to display (mdi icon) */
    icon: string;
    /** Main title text */
    title: string;
    /** Description text */
    description: string;
    /** Action button text (if any) */
    actionText?: string;
    /** Action button click handler */
    onaction?: () => void;
    /** Whether to show icon in a circular background */
    showIconCircle?: boolean;
    /** Additional content via snippet */
    children?: Snippet;
  }

  let { icon, title, description, actionText, onaction, showIconCircle = false, children }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-3 px-6 py-15 text-center text-foreground-muted">
  {#if showIconCircle}
    <div class="bg-muted/50 flex h-20 w-20 items-center justify-center rounded-full">
      <Icon {icon} width="48" class="text-foreground-muted/50" />
    </div>
  {:else}
    <Icon {icon} width="56" />
  {/if}

  <h3 class="m-0 text-lg font-semibold text-foreground">{title}</h3>
  <p class="m-0 max-w-sm text-sm">{description}</p>

  {#if children}
    {@render children()}
  {/if}

  {#if actionText && onaction}
    <button type="button" class={button({ variant: "primary", size: "md", class: "mt-2" })} onclick={onaction}>
      {actionText}
    </button>
  {/if}
</div>
