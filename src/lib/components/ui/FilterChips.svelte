<script lang="ts" module>
  export interface FilterChip<Id extends string> {
    id: Id;
    label: string;
    count?: number;
    tone?: "error";
  }
</script>

<script lang="ts" generics="T extends string">
  interface Props {
    options: FilterChip<T>[];
    value: T;
    label?: string;
    onchange: (value: T) => void;
  }

  let { options, value, label = "Show", onchange }: Props = $props();
</script>

<div class="flex flex-wrap gap-1" role="group" aria-label={label}>
  {#each options as o (o.id)}
    {@const active = o.id === value}
    <button
      type="button"
      class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md px-2.5 text-[13px] hover:bg-muted {active
        ? 'bg-muted font-semibold'
        : ''} {o.tone === 'error'
        ? 'text-error'
        : active
          ? 'text-foreground'
          : 'text-foreground-muted hover:text-foreground'}"
      aria-pressed={active}
      onclick={() => onchange(o.id)}
    >
      {o.label}
      {#if o.count !== undefined}
        <span class="text-xs text-foreground-subtle tabular-nums">{o.count}</span>
      {/if}
    </button>
  {/each}
</div>
