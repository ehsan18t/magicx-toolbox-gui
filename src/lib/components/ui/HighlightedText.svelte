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

  const { text, ranges = [], class: className = "", highlightClass = "" }: Props = $props();

  /** Build text segments from ranges */
  const segments = $derived.by(() => {
    if (!ranges || ranges.length === 0 || !text) {
      return [{ text, highlighted: false }];
    }

    const result: Array<{ text: string; highlighted: boolean }> = [];
    let lastEnd = 0;

    // Process range pairs
    for (let i = 0; i < ranges.length; i += 2) {
      const start = ranges[i];
      const end = ranges[i + 1];

      // Validate range
      if (start < 0 || end > text.length || start >= end) {
        continue;
      }

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
  >{#each segments as segment, i (`highlight-text-seg-${i}`)}{#if segment.highlighted}<mark class={highlightClass}
        >{segment.text}</mark
      >{:else}{segment.text}{/if}{/each}</span
>
