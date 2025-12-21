<script lang="ts">
  interface Props {
    value: number;
    max?: number;
    size?: "sm" | "md" | "lg";
    variant?: "default" | "success" | "warning" | "error";
    showLabel?: boolean;
    class?: string;
  }

  let {
    value,
    max = 100,
    size = "md",
    variant = "default",
    showLabel = false,
    class: className = "",
  }: Props = $props();

  const percentage = $derived(Math.min(100, Math.max(0, (value / max) * 100)));

  const sizeClasses: Record<string, string> = {
    sm: "h-1.5",
    md: "h-2.5",
    lg: "h-4",
  };

  const variantClasses: Record<string, string> = {
    default: "bg-accent",
    success: "bg-success",
    warning: "bg-warning",
    error: "bg-error",
  };
</script>

<div class="flex w-full items-center gap-3 {className}">
  <div
    class="bg-muted relative flex-1 overflow-hidden rounded-full {sizeClasses[size]}"
    role="progressbar"
    aria-valuenow={Math.round(percentage)}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label="Progress"
  >
    <div
      class="absolute inset-y-0 left-0 rounded-full transition-all duration-300 ease-out {variantClasses[variant]}"
      style="width: {percentage}%"
    ></div>
  </div>
  {#if showLabel}
    <span class="min-w-12 text-right text-sm font-medium text-foreground-muted tabular-nums">
      {Math.round(percentage)}%
    </span>
  {/if}
</div>
