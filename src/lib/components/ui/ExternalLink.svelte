<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { openExternal } from "$lib/utils/externalLink";
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { link, type LinkVariants } from "./variants";

  // Without `variant` the anchor carries only `class`, e.g. an icon button's look.
  interface Props extends Omit<HTMLAnchorAttributes, "href" | "class" | "children">, LinkVariants {
    href: string;
    tooltip?: string;
    class?: string;
    children: Snippet;
  }

  let { href, tooltip, variant, tone, class: className, children, ...rest }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- an external URL, which resolve() (app routes) does not apply to -->
<a
  {href}
  class={variant ? link({ variant, tone, class: className }) : className}
  {...rest}
  onclick={(event) => openExternal(event, href)}
  use:tooltipAction={tooltip}
>
  {@render children()}
</a>
