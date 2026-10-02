<script lang="ts">
  interface Props {
    text: string;
    /** uFuzzy match ranges, flat: [start, end, start, end, …]. */
    ranges?: number[];
    class?: string;
    highlightClass?: string;
  }

  let {
    text,
    ranges = [],
    class: className,
    highlightClass = "rounded-sm bg-accent/25 text-foreground",
  }: Props = $props();

  const segments = $derived.by(() => {
    const spans: { start: number; end: number }[] = [];
    for (let i = 0; i < ranges.length; i += 2) {
      const start = ranges[i];
      const end = ranges[i + 1];
      if (start >= 0 && end <= text.length && start < end) spans.push({ start, end });
    }
    spans.sort((a, b) => a.start - b.start);

    const merged: { start: number; end: number }[] = [];
    for (const span of spans) {
      const last = merged.at(-1);
      if (last && span.start <= last.end) last.end = Math.max(last.end, span.end);
      else merged.push(span);
    }

    const result: { text: string; highlighted: boolean }[] = [];
    let cursor = 0;
    for (const { start, end } of merged) {
      if (start > cursor) result.push({ text: text.slice(cursor, start), highlighted: false });
      result.push({ text: text.slice(start, end), highlighted: true });
      cursor = end;
    }
    if (cursor < text.length || result.length === 0) result.push({ text: text.slice(cursor), highlighted: false });
    return result;
  });
</script>

<span class={className}
  >{#each segments as segment, i (i)}{#if segment.highlighted}<mark class={highlightClass}>{segment.text}</mark
      >{:else}{segment.text}{/if}{/each}</span
>
