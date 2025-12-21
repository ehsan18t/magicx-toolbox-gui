<script lang="ts">
  import { Icon } from "$lib/components/shared";

  type Size = "sm" | "md" | "lg";

  interface Props {
    size?: Size;
    class?: string;
    "aria-label"?: string;
  }

  const { size = "md", class: className = "", "aria-label": ariaLabel = "Loading" }: Props = $props();

  const sizeMap: Record<Size, number> = {
    sm: 16,
    md: 24,
    lg: 32,
  };
</script>

<span role="status" aria-label={ariaLabel} class="inline-flex {className}">
  <Icon icon="mdi:loading" width={sizeMap[size]} class="animate-spin text-accent" />
</span>
