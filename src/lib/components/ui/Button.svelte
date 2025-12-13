<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "secondary" | "ghost" | "danger" | "warning" | "success";
  type Size = "sm" | "md" | "lg";

  interface Props extends HTMLButtonAttributes {
    variant?: Variant;
    size?: Size;
    loading?: boolean;
    class?: string;
    children: Snippet;
  }

  const {
    variant = "secondary",
    size = "md",
    loading = false,
    disabled,
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const baseClasses =
    "inline-flex items-center justify-center gap-2 rounded-lg border-0 font-medium transition-all duration-150 cursor-pointer disabled:cursor-not-allowed disabled:opacity-60";

  const variantClasses: Record<Variant, string> = {
    primary: "bg-accent text-accent-foreground hover:bg-accent-hover",
    secondary: "bg-muted text-foreground hover:bg-muted/80",
    ghost: "bg-transparent text-foreground-muted hover:bg-muted hover:text-foreground",
    danger: "bg-error text-white hover:bg-error/90",
    warning: "bg-warning text-black hover:bg-warning/90",
    success: "bg-success text-white hover:bg-success/90",
  };

  const sizeClasses: Record<Size, string> = {
    sm: "px-3 py-1.5 text-xs",
    md: "px-4 py-2 text-sm",
    lg: "px-5 py-2.5 text-base",
  };
</script>

<button
  class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
  disabled={disabled || loading}
  {...rest}
>
  {#if loading}
    <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      ></path>
    </svg>
  {/if}
  {@render children()}
</button>
