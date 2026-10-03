<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import IconTile from "./IconTile.svelte";
  import Spinner from "./Spinner.svelte";

  type Props = {
    title?: string;
    description: string;
    action?: { label: string; onclick: () => void };
    children?: Snippet;
  } & ({ icon: IconName; loading?: false; showIconCircle?: boolean } | { icon?: never; loading: true });

  let { title, description, action, children, ...visual }: Props = $props();
</script>

<div
  class="flex animate-fade-in flex-col items-center justify-center gap-2 px-6 py-14 text-center text-foreground-muted"
>
  {#if visual.loading}
    <Spinner size="lg" label={description} class="mb-1 text-foreground-muted" />
  {:else if visual.showIconCircle}
    <IconTile icon={visual.icon} size="2xl" shape="circle" tone="neutral" class="mb-1" />
  {:else}
    <Icon icon={visual.icon} size="6xl" class="mb-1" />
  {/if}

  {#if title}<h3 class="m-0 text-base font-semibold text-foreground">{title}</h3>{/if}
  <p class="m-0 max-w-sm text-ui wrap-break-word">{description}</p>

  {@render children?.()}

  {#if action}
    <Button variant="primary" class="mt-2" onclick={action.onclick}>{action.label}</Button>
  {/if}
</div>
