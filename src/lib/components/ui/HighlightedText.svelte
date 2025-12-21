<!--
  HighlightedText.svelte

  Renders text with highlighted match ranges from uFuzzy search.
  Uses semantic <mark> elements for accessibility.

  Props:
  - text: The full text string
  - ranges: Array of [start, end, start, end, ...] indices to highlight
  - class: Optional CSS classes for the container
  - highlightClass: CSS classes for the <mark> elements
-->
<script lang="ts">
  interface Props {
    /** The text to render */
    text: string;
    /** Highlight ranges: [start, end, start, end, ...] */
    ranges?: number[];
    /** Optional container class */
    class?: string;
    /** CSS class for highlight marks */
    highlightClass?: string;
  }

  let { text, ranges = [], class: className = "", highlightClass = "" }: Props = $props();

  /** Build text segments from ranges */
  const segments = $derived.by(() => {
    if (!ranges || ranges.length === 0 || !text) {
      return [{ text, highlighted: false }];
    }

    // Sanitize ranges: Sort and merge overlapping intervals
    // uFuzzy usually returns sorted non-overlapping ranges, but for robustness
    // (and to satisfy code review), we normalize them here.
    const sortedRanges: Array<{ start: number; end: number }> = [];

    // 1. Convert flat array to objects
    for (let i = 0; i < ranges.length; i += 2) {
      const start = ranges[i];
      const end = ranges[i + 1];
      if (start >= 0 && end <= text.length && start < end) {
        sortedRanges.push({ start, end });
      }
    }

    // 2. Sort by start position
    sortedRanges.sort((a, b) => a.start - b.start);

    // 3. Merge overlaps
    const mergedRanges: Array<{ start: number; end: number }> = [];
    if (sortedRanges.length > 0) {
      let current = sortedRanges[0];

      for (let i = 1; i < sortedRanges.length; i++) {
        const next = sortedRanges[i];
        if (next.start <= current.end) {
          // Overlap or adjacent - merge
          current.end = Math.max(current.end, next.end);
        } else {
          // No overlap - push current and start new
          mergedRanges.push(current);
          current = next;
        }
      }
      mergedRanges.push(current);
    }

    const result: Array<{ text: string; highlighted: boolean }> = [];
    let lastEnd = 0;

    // Process merged range pairs
    for (const { start, end } of mergedRanges) {
      // Add non-highlighted segment before this match
      if (start > lastEnd) {
        result.push({
          text: text.slice(lastEnd, start),
          highlighted: false,
        });
      }

      // Add highlighted segment
      result.push({
        text: text.slice(start, end),
        highlighted: true,
      });

      lastEnd = end;
    }

    // Add remaining text after last match
    if (lastEnd < text.length) {
      result.push({
        text: text.slice(lastEnd),
        highlighted: false,
      });
    }

    return result.length > 0 ? result : [{ text, highlighted: false }];
  });
</script>

<span class={className}
  >{#each segments as segment, i (`highlight-seg-${i}-${segment.highlighted}-${segment.text.slice(0, 8)}`)}{#if segment.highlighted}<mark
        class={highlightClass}>{segment.text}</mark
      >{:else}{segment.text}{/if}{/each}</span
>
