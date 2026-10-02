<script lang="ts">
  import { cn } from "$lib/utils/cn";

  interface Props {
    value: number;
    max: number;
    label: string;
    class?: string;
  }

  let { value, max, label, class: className }: Props = $props();
</script>

<span
  role="meter"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={max}
  aria-valuenow={value}
  class={cn("block h-1 overflow-hidden rounded-full bg-muted", className)}
>
  <span
    class="block h-full rounded-full bg-accent transition-[width] duration-slower ease-out"
    style:width="{max > 0 ? (value / max) * 100 : 0}%"
  ></span>
</span>
