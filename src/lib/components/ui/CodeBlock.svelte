<script lang="ts" module>
  const KIND = {
    /** One script command, wrapping anywhere. */
    command: { tag: "code", look: "block bg-background px-3 py-2 break-all text-foreground-soft" },
    /** Scrolling output. */
    log: { tag: "pre", look: "m-0 max-h-72 overflow-auto bg-surface p-3 leading-relaxed text-foreground" },
  };
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLAttributes<HTMLElement>, "class"> {
    kind: keyof typeof KIND;
    children: Snippet;
  }

  let { kind, children, ...rest }: Props = $props();
</script>

<svelte:element
  this={KIND[kind].tag}
  class={cn("rounded-md border border-border font-mono text-caption whitespace-pre-wrap", KIND[kind].look)}
  {...rest}>{@render children()}</svelte:element
>
