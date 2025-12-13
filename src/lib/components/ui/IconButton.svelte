<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Icon from "../Icon.svelte";

  interface Props extends Omit<HTMLButtonAttributes, "class"> {
    icon: string;
    size?: number;
    class?: string;
  }

  const { icon, size = 20, class: className = "", ...rest }: Props = $props();
</script>

<button
  type="button"
  class="hover:bg-muted flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-foreground-muted transition-colors hover:text-foreground {className}"
  {...rest}
>
  <Icon {icon} width={size} />
</button>
