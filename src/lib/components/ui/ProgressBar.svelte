<script lang="ts">
  interface Props {
    value: number;
    label: string;
    max?: number;
    showValue?: boolean;
  }

  let { value, label, max = 100, showValue = false }: Props = $props();

  const percent = $derived(Math.min(100, Math.max(0, (value / max) * 100)));
</script>

<div class="flex w-full items-center gap-3">
  <div
    class="relative h-4 flex-1 overflow-hidden rounded-full bg-muted"
    role="progressbar"
    aria-valuenow={Math.round(percent)}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label={label}
  >
    <div
      class="absolute inset-y-0 left-0 rounded-full bg-accent transition-[width] duration-slower ease-out"
      style:width="{percent}%"
    ></div>
  </div>
  {#if showValue}
    <span class="min-w-12 text-right text-sm font-medium text-foreground-muted tabular-nums"
      >{Math.round(percent)}%</span
    >
  {/if}
</div>
