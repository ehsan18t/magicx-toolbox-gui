<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    title: string;
    icon: string;
    variant?: "default" | "danger";
    onclick: () => void;
  }

  let { title, icon, variant = "default", onclick }: Props = $props();
</script>

<!-- Windows 11 caption button: 46px wide, full bar height, red close hover. -->
<button
  class="flex h-full w-11.5 cursor-default items-center justify-center text-foreground {variant === 'danger'
    ? 'hover:bg-[#c42b1c] hover:text-white active:bg-[#c42b1c]/90'
    : 'hover:bg-muted active:bg-foreground/5'}"
  type="button"
  aria-label={title}
  {title}
  {onclick}
>
  <Icon {icon} width="16" />
</button>
