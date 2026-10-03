<script lang="ts">
  import { SEP } from "$lib/utils/format";

  interface Props {
    /** Rendered as "12 starred · 3 applied". */
    items: { value: number; label: string }[];
  }

  let { items }: Props = $props();
</script>

<p class="m-0 text-xs text-foreground-muted">
  {#each items as { value, label }, i (label)}{i > 0 ? SEP : ""}<span class="font-semibold text-foreground tabular-nums"
      >{value}</span
    >{` ${label}`}{/each}
</p>
