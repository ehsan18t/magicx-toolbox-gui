<script lang="ts">
  import { Icon, type IconName } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import { ICON_SIZE } from "./icon";
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
    <div class="mb-1 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
      <Icon icon={visual.icon} width={ICON_SIZE["2xl"]} />
    </div>
  {:else}
    <Icon icon={visual.icon} width={ICON_SIZE["3xl"]} class="mb-1" />
  {/if}

  {#if title}<h3 class="m-0 text-base font-semibold text-foreground">{title}</h3>{/if}
  <p class="m-0 max-w-sm text-ui wrap-break-word">{description}</p>

  {@render children?.()}

  {#if action}
    <Button variant="primary" class="mt-2" onclick={action.onclick}>{action.label}</Button>
  {/if}
</div>
