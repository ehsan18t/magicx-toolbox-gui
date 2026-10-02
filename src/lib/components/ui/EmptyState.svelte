<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";

  type Props = {
    title?: string;
    description: string;
    actionText?: string;
    onaction?: () => void;
    children?: Snippet;
  } & ({ icon: string; loading?: false; showIconCircle?: boolean } | { icon?: never; loading: true });

  let { title, description, actionText, onaction, children, ...visual }: Props = $props();
</script>

<div
  class="flex animate-fade-in flex-col items-center justify-center gap-2 px-6 py-14 text-center text-foreground-muted"
>
  {#if visual.loading}
    <Icon icon="mdi:loading" width="40" class="mb-1 animate-spin" />
  {:else if visual.showIconCircle}
    <div class="mb-1 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
      <Icon icon={visual.icon} width="32" />
    </div>
  {:else}
    <Icon icon={visual.icon} width="40" class="mb-1" />
  {/if}

  {#if title}<h3 class="m-0 text-base font-semibold text-foreground">{title}</h3>{/if}
  <p class="m-0 max-w-sm text-ui wrap-break-word">{description}</p>

  {@render children?.()}

  {#if actionText && onaction}
    <Button variant="primary" class="mt-2" onclick={onaction}>{actionText}</Button>
  {/if}
</div>
