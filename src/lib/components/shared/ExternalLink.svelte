<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { open } from "@tauri-apps/plugin-shell";
  import type { Snippet } from "svelte";

  interface Props {
    href: string;
    children: Snippet;
    title?: string;
    class?: string;
    [key: string]: unknown;
  }

  let { href, children, title, ...rest }: Props = $props();

  async function handleClick(event: MouseEvent) {
    // Only intercept external links
    if (href && (href.startsWith("http://") || href.startsWith("https://"))) {
      event.preventDefault();
      try {
        await open(href);
      } catch (error) {
        console.error(`Failed to open external link: ${href}`, error);
      }
    }
  }
</script>

<!-- This is an external link component that opens URLs in the system browser via Tauri shell -->
<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
<a {href} {...rest} onclick={handleClick} use:tooltip={title}>
  {@render children()}
</a>
