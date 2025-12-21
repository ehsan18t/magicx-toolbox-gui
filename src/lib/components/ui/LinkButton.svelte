<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import type { Snippet } from "svelte";
  import { linkButton, type LinkButtonVariants } from "./variants";

  /**
   * LinkButton - Button styled to look like a link button.
   * For actual navigation links, use ExternalLink component instead.
   */

  interface Props {
    /** Button size */
    size?: LinkButtonVariants["size"];
    /** Icon name (mdi icon) */
    icon?: string;
    /** Additional classes */
    class?: string;
    /** Button content */
    children: Snippet;
    /** Rest props */
    [key: string]: unknown;
  }

  const { size = "md", icon, class: className = "", children, ...rest }: Props = $props();
</script>

<button type="button" class={linkButton({ size, class: className })} {...rest}>
  {#if icon}
    <Icon {icon} width="18" />
  {/if}
  {@render children()}
</button>
