<script lang="ts">
  import { ICON_SIZE } from "$lib/components/ui/icon";
  import { type IconName, iconRegistry } from "./icons";

  interface Props {
    icon: IconName;
    width?: number | string;
    height?: number | string;
    class?: string;
  }

  let { icon, width = ICON_SIZE.xl, height, class: className }: Props = $props();

  const IconComponent = $derived(iconRegistry[icon]);
</script>

<IconComponent {width} height={height ?? width} class={className} />
