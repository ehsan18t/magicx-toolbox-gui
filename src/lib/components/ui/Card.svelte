<script lang="ts">
  import type { Snippet } from "svelte";

  type Variant = "default" | "elevated" | "outlined" | "ghost";

  interface Props {
    variant?: Variant;
    hover?: boolean;
    padding?: "none" | "sm" | "md" | "lg";
    class?: string;
    children: Snippet;
  }

  const { variant = "default", hover = false, padding = "md", class: className = "", children }: Props = $props();

  const baseClasses = "rounded-lg border transition-all duration-200";

  const variantClasses: Record<Variant, string> = {
    default: "border-border bg-card",
    elevated: "border-border bg-elevated shadow-md",
    outlined: "border-border bg-transparent",
    ghost: "border-transparent bg-transparent",
  };

  const hoverClasses = $derived(hover ? "hover:border-border-hover hover:shadow-md" : "");

  const paddingClasses: Record<string, string> = {
    none: "",
    sm: "p-2",
    md: "p-4",
    lg: "p-6",
  };
</script>

<div class="{baseClasses} {variantClasses[variant]} {hoverClasses} {paddingClasses[padding]} {className}">
  {@render children()}
</div>
