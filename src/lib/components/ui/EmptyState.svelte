<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import { button } from "./variants";

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

<div
  class="flex animate-fade-in flex-col items-center justify-center gap-2 px-6 py-14 text-center text-foreground-muted"
>
  {#if showIconCircle}
    <div class="mb-1 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
      <Icon {icon} width="32" />
    </div>
  {:else}
    <Icon {icon} width="40" class="mb-1 {icon === 'mdi:loading' ? 'animate-spin' : ''}" />
  {/if}

  {#if title}<h3 class="m-0 text-base font-semibold text-foreground">{title}</h3>{/if}
  <p class="m-0 max-w-sm text-[13px] wrap-break-word">{description}</p>

  {#if children}
    {@render children()}
  {/if}

  {#if actionText && onaction}
    <button type="button" class={button({ variant: "primary", size: "md", class: "mt-2" })} onclick={onaction}>
      {actionText}
    </button>
  {/if}
</div>
