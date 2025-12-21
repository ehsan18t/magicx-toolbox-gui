<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";

  interface Props {
    icon: string;
    label: string;
    title: string;
    children?: import("svelte").Snippet;
    headerExtra?: import("svelte").Snippet;
  }

  const { icon, label, title, children, headerExtra }: Props = $props();
</script>

<div class="flex items-center gap-3 rounded-xl border border-border bg-card p-3">
  <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
    <Icon {icon} width="20" class="text-accent" />
  </div>
  <div class="flex min-w-0 flex-1 flex-col">
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium text-foreground-muted uppercase">
        {label}
      </span>
      {#if headerExtra}
        {@render headerExtra()}
      {/if}
    </div>
    <h3 class="line-clamp-1 text-sm font-semibold text-foreground" use:tooltip={title}>
      {title}
    </h3>
    {#if children}
      <div class="mt-0.5 flex w-full min-w-0 items-center gap-2 text-xs text-foreground-muted">
        {@render children()}
      </div>
    {/if}
  </div>
</div>
