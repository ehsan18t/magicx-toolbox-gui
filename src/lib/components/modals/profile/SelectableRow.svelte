<script lang="ts">
  import { Checkbox } from "$lib/components/ui";
  import { DISABLED } from "$lib/components/ui/variants";
  import type { Snippet } from "svelte";

  interface Props {
    checked: boolean;
    /** The accessible name, so trailing badges stay out of it. */
    label: string;
    disabled?: boolean;
    onclick: () => void;
    trailing?: Snippet;
    children: Snippet;
  }

  let { checked, label, disabled = false, onclick, trailing, children }: Props = $props();
</script>

<button
  type="button"
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  {onclick}
  class={[
    "flex w-full cursor-pointer items-center gap-3 px-3 py-2.5 text-left transition-colors focus-visible:bg-muted focus-visible:-outline-offset-2 enabled:hover:bg-muted",
    DISABLED,
  ]}
>
  <Checkbox {checked} />
  <span class="min-w-0 flex-1">{@render children()}</span>
  {@render trailing?.()}
</button>
