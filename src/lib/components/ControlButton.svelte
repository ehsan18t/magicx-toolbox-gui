<script lang="ts">
  import Icon from "@iconify/svelte";

  type Variant = "default" | "theme" | "danger";

  interface Props {
    title: string;
    icon: string;
    variant?: Variant;
    onClick: () => void;
  }

  const { title, icon, variant = "default", onClick }: Props = $props();

  const variantClasses: Record<Variant, string> = {
    default: "hover:bg-foreground/8 active:bg-foreground/12 focus-visible:bg-foreground/12",
    theme: "hover:bg-accent/12 active:bg-accent/18 focus-visible:bg-accent/18",
    danger: "hover:bg-error/12 active:bg-error/18 focus-visible:bg-error/18",
  };

  const iconClasses: Record<Variant, string> = {
    default: "text-foreground-muted group-hover:text-foreground",
    theme: "text-foreground-muted group-hover:text-accent",
    danger: "text-foreground-muted group-hover:text-error",
  };

  // Derived classes based on variant
  const btnClass = $derived(variantClasses[variant]);
  const iconClass = $derived(iconClasses[variant]);
</script>

<button
  class="group relative flex h-8 w-8 cursor-pointer items-center justify-center overflow-hidden rounded border-0 bg-transparent transition-colors duration-150 outline-none active:scale-90 disabled:pointer-events-none disabled:cursor-default disabled:opacity-40 {btnClass}"
  {title}
  onclick={onClick}
>
  <span
    class="relative z-10 flex items-center justify-center transition-transform duration-150 {iconClass}"
  >
    <Icon {icon} width="16" height="16" />
  </span>
</button>
