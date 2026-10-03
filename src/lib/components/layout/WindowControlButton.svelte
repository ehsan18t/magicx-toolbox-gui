<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { CAPTION_BUTTON } from "$lib/design";

  interface Props {
    label: string;
    glyph: "minimize" | "maximize" | "restore" | "close";
    onclick: () => void;
  }

  let { label, glyph, onclick }: Props = $props();
</script>

<button
  class="flex h-8 w-10 cursor-default items-center justify-center rounded-md text-foreground-muted {CAPTION_BUTTON[
    glyph === 'close' ? 'close' : 'other'
  ]}"
  type="button"
  aria-label={label}
  use:tooltip={label}
  {onclick}
>
  <!-- 1px line glyphs on a 10px grid, drawn like the Windows 11 caption icons. -->
  <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" aria-hidden="true">
    {#if glyph === "minimize"}
      <path d="M0 5.5h10" />
    {:else if glyph === "maximize"}
      <rect x="0.5" y="0.5" width="9" height="9" rx="1.5" />
    {:else if glyph === "restore"}
      <rect x="0.5" y="2.5" width="7" height="7" rx="1.5" />
      <path d="M2.5 0.5h5.5a1.5 1.5 0 0 1 1.5 1.5v5.5" />
    {:else}
      <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />
    {/if}
  </svg>
</button>
