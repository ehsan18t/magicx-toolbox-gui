<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import IconButton from "./IconButton.svelte";
  import ModalTitle, { type ModalTitleSize } from "./ModalTitle.svelte";
  import { FLOATING_CLOSE } from "./variants";

  interface Props {
    title: string;
    size?: ModalTitleSize;
    /** Before the title, e.g. an icon. */
    leading?: Snippet;
    /** Adds a close button. */
    onclose?: () => void;
    /** Pins the close button to the dialog's corner instead of the title row. */
    floating?: boolean;
    class?: string;
    /** Under the title, e.g. a subtitle. */
    children?: Snippet;
  }

  let { title, size, leading, onclose, floating = false, class: className, children }: Props = $props();
</script>

<div class={cn("flex shrink-0 items-start justify-between gap-3 px-6 pt-5 pb-3", floating && "relative", className)}>
  <div class="flex min-w-0 items-center gap-3">
    {@render leading?.()}
    <div class="min-w-0">
      <ModalTitle {size}>{title}</ModalTitle>
      {@render children?.()}
    </div>
  </div>
  {#if onclose}
    <IconButton icon="mdi:close" label="Close" class={floating ? FLOATING_CLOSE : undefined} onclick={onclose} />
  {/if}
</div>
