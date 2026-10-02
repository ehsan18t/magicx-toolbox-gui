<script lang="ts" module>
  const ICON_SIZE = { sm: 16, md: 24 } as const;
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";

  interface Props {
    size?: keyof typeof ICON_SIZE;
    label?: string;
  }

  let { size = "md", label = "Loading" }: Props = $props();
</script>

<span role="status" aria-label={label} class="inline-flex">
  <Icon icon="mdi:loading" width={ICON_SIZE[size]} class="animate-spin text-accent" />
</span>
