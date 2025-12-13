<script lang="ts">
  import type { Snippet } from "svelte";

  type Variant = "default" | "accent" | "success" | "warning" | "error" | "info";
  type Size = "sm" | "md";

  interface Props {
    variant?: Variant;
    size?: Size;
    class?: string;
    children: Snippet;
  }

  const { variant = "default", size = "sm", class: className = "", children }: Props = $props();

  const baseClasses = "inline-flex items-center gap-1 rounded-md font-semibold tracking-wide uppercase";

  const variantClasses: Record<Variant, string> = {
    default: "bg-muted text-foreground-muted",
    accent: "bg-accent/15 text-accent",
    success: "bg-success/15 text-success",
    warning: "bg-warning/15 text-warning",
    error: "bg-error/15 text-error",
    info: "bg-info/15 text-info",
  };

  const sizeClasses: Record<Size, string> = {
    sm: "px-1.5 py-0.5 text-[9px]",
    md: "px-2 py-1 text-[10px]",
  };
</script>

<span class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}">
  {@render children()}
</span>
