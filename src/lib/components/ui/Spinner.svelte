<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { IconSize } from "$lib/design";
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";

  interface Props {
    /** On the icon scale. */
    size?: IconSize;
    /** `current` takes the surrounding text colour, e.g. inside a button. */
    tone?: "accent" | "current";
    /** Names the status; without it or `children` the spinner is decorative. */
    label?: string;
    class?: string;
    /** Visible status text beside the spinner. */
    children?: Snippet;
  }

  let { size = "3xl", tone = "accent", label, class: className, children }: Props = $props();

  const isStatus = $derived(!!label || !!children);
</script>

<span
  role={isStatus ? "status" : undefined}
  aria-label={label}
  aria-hidden={isStatus ? undefined : "true"}
  class={cn("inline-flex", children && "items-center gap-2", className)}
>
  <Icon icon="mdi:loading" {size} class={cn("animate-spin", tone === "accent" && "text-accent")} />
  {@render children?.()}
</span>
