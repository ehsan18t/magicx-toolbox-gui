<script lang="ts">
  import { tooltip as tooltipAction } from "$lib/actions/tooltip";
  import { openExternal } from "$lib/utils/externalLink";
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLAnchorAttributes, "href" | "children"> {
    href: string;
    tooltip?: string;
    children: Snippet;
  }

  let { href, tooltip, children, ...rest }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
<a {href} {...rest} onclick={(event) => openExternal(event, href)} use:tooltipAction={tooltip}>
  {@render children()}
</a>
