<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    title: string;
    commands: string[];
    icon?: string;
    iconClass?: string;
  }

  const { title, commands, icon = "mdi:console", iconClass = "text-foreground-muted" }: Props = $props();
</script>

{#if commands.length > 0}
  <div class="space-y-2">
    <h5 class="flex items-center gap-1 text-xs font-medium text-foreground-muted">
      <Icon {icon} width="14" class={iconClass} />
      {title}
    </h5>
    <div class="space-y-1">
      {#each commands as cmd, idx (idx)}
        <div class="rounded-lg border border-border/60 bg-background px-3 py-2">
          <code class="bg-transparent p-0 font-mono text-[10px] break-all text-foreground">
            {cmd}
          </code>
        </div>
      {/each}
    </div>
  </div>
{/if}
