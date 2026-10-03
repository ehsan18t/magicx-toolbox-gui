<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { HEADING, type IconName } from "$lib/design";
  import Button from "./Button.svelte";
  import IconTile from "./IconTile.svelte";
  import Spinner from "./Spinner.svelte";

  type Props = {
    title?: string;
    description: string;
    action?: { label: string; onclick: () => void };
  } & ({ icon: IconName; loading?: false; showIconCircle?: boolean } | { icon?: never; loading: true });

  let { title, description, action, ...visual }: Props = $props();
</script>

<div
  class="flex animate-fade-in flex-col items-center justify-center gap-2 px-6 py-14 text-center text-foreground-muted"
>
  {#if visual.loading}
    <Spinner size="6xl" tone="current" class="mb-1" />
  {:else if visual.showIconCircle}
    <IconTile icon={visual.icon} size="2xl" shape="circle" tone="neutral" class="mb-1" />
  {:else}
    <Icon icon={visual.icon} size="6xl" class="mb-1" />
  {/if}

  {#if title}<h2 class={["m-0 text-foreground", HEADING.section]}>{title}</h2>{/if}
  <p class="m-0 max-w-sm text-ui wrap-break-word" role={visual.loading ? "status" : undefined}>{description}</p>

  {#if action}
    <Button variant="primary" class="mt-2" onclick={action.onclick}>{action.label}</Button>
  {/if}
</div>
