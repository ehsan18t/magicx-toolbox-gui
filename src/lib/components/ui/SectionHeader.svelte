<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { counterBadge, sectionHeading } from "./variants";

  /**
   * SectionHeader - Consistent section heading with optional icon and count
   *
   * Used for section titles in modals and panels.
   */

  interface Props {
    /** Icon name (mdi icon) */
    icon?: string;
    /** Heading text */
    title: string;
    /** Optional count badge */
    count?: number;
    /** Size variant */
    size?: "xs" | "sm" | "md";
    /** Whether title is uppercase */
    uppercase?: boolean;
    /** Additional class names */
    class?: string;
  }

  let { icon, title, count, size = "sm", uppercase = false, class: className = "" }: Props = $props();
</script>

<h3
  class={sectionHeading({
    size,
    uppercase,
    class: className,
  })}
>
  {#if icon}
    <Icon {icon} width={size === "xs" ? "12" : size === "sm" ? "14" : "16"} />
  {/if}
  <span>{title}</span>
  {#if count !== undefined}
    <span class={counterBadge({ size: "sm", variant: "muted" })}>{count}</span>
  {/if}
</h3>
