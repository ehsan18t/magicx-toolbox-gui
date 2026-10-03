<script lang="ts">
  import { ICON_SIZE, type IconName, type IconSize, iconRegistry } from "$lib/design";

  interface Props {
    icon: IconName;
    size?: IconSize;
    class?: string | undefined;
  }

  let { icon, size = "3xl", class: className }: Props = $props();

  const IconComponent = $derived(iconRegistry[icon]);

  $effect(() => {
    // A name from data (tweak YAML, stored state) can miss the registry and would render nothing.
    if (import.meta.env.DEV && !IconComponent) console.warn(`Icon "${icon}" is not registered in $lib/design/icons.ts`);
  });
</script>

{#if IconComponent}
  <IconComponent width={ICON_SIZE[size]} height={ICON_SIZE[size]} class={className} aria-hidden="true" />
{/if}
